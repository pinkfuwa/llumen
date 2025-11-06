use std::sync::Arc;

use anyhow::Context as _;
use entity::{ChunkKind, MessageKind, chat, chunk, message, model, user};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, QueryFilter,
};
use tokio::join;
use tokio_stream::{Stream, StreamExt};

use super::tools::{CrawlTool, LuaReplTool, WebSearchTool};
use super::{
    channel::{self, Publisher},
    prompt::Prompt,
    token::Token,
};
use crate::chat::deep_prompt::{DeepPrompt, PromptContext};
use crate::{chat::prompt::PromptKind, openrouter, utils::blob::BlobDB};

#[derive(Debug, Clone, Copy)]
pub enum StreamEndReason {
    Halt,
    Exhausted,
}

/// The global context for the chat system.
/// It holds the database connection, the OpenRouter client, and the channel context.
pub struct Context {
    pub(super) db: DatabaseConnection,
    pub(super) openrouter: openrouter::Openrouter,
    pub(super) channel: Arc<channel::Context<Token>>,
    pub(super) prompt: Prompt,
    pub(super) blob: Arc<BlobDB>,
    pub(super) web_search_tool: Arc<WebSearchTool>,
    pub(super) crawl_tool: Arc<CrawlTool>,
    pub(super) lua_repl_tool: Arc<LuaReplTool>,
    pub(super) deep_prompt: Arc<DeepPrompt>,
}

impl Context {
    // TODO: put API Key in main
    pub fn new(
        db: DatabaseConnection,
        openrouter: openrouter::Openrouter,
        blob: Arc<BlobDB>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db,
            openrouter,
            channel: Arc::new(channel::Context::new()),
            prompt: Prompt::new(),
            blob,
            web_search_tool: Arc::new(WebSearchTool::new()),
            crawl_tool: Arc::new(CrawlTool::new()),
            lua_repl_tool: Arc::new(LuaReplTool::new()),
            deep_prompt: Arc::new(DeepPrompt::new()),
        })
    }

    pub fn get_completion_context(
        self: &Arc<Self>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> impl std::future::Future<Output = Result<CompletionContext, anyhow::Error>> + '_ {
        CompletionContext::new(self.clone(), user_id, chat_id, model_id)
    }
    pub fn halt_completion(&self, chat_id: i32) {
        self.channel.stop(chat_id)
    }
    pub fn subscribe(self: Arc<Self>, chat_id: i32) -> impl Stream<Item = Token> + 'static {
        self.channel.clone().subscribe(chat_id)
    }
    pub fn is_streaming(&self, chat_id: i32) -> bool {
        !self.channel.publishable(chat_id)
    }
}

/// Creates a new error chunk with the given message.
fn error_chunk(msg: String, message_id: i32) -> chunk::ActiveModel {
    chunk::ActiveModel {
        message_id: ActiveValue::Set(message_id),
        content: ActiveValue::Set(msg),
        kind: ActiveValue::Set(ChunkKind::Error),
        ..Default::default()
    }
}

/// The context for a single completion request.
///
/// It holds the state for the completion, including the model, chat, message, chunks, tokens, and user.
pub struct CompletionContext {
    /// The model used for the completion.
    pub(super) model: model::Model,
    /// The chat the completion belongs to.
    pub(super) chat: chat::ActiveModel,
    /// The message the completion belongs to.
    pub(super) message: message::ActiveModel,
    /// New chunks generated during the completion.
    pub(super) new_chunks: Vec<chunk::ActiveModel>,
    /// The previous chunks in the chat.
    pub(super) messages_with_chunks: Vec<(message::Model, Vec<chunk::Model>)>,
    /// The user who initiated the completion.
    pub(super) user: user::Model,
    /// The publisher for the completion's tokens.
    publisher: Publisher<Token>,
    /// The global context.
    ctx: Arc<Context>,
}

impl CompletionContext {
    /// Creates a new completion context.
    pub async fn new(
        ctx: Arc<Context>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> Result<Self, anyhow::Error> {
        let db = &ctx.db;

        let (user, chat, model) = join!(
            user::Entity::find_by_id(user_id).one(db),
            chat::Entity::find_by_id(chat_id)
                .filter(chat::Column::OwnerId.eq(user_id))
                .one(db),
            model::Entity::find_by_id(model_id).one(db)
        );

        let user = user?.ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let chat = chat?.ok_or_else(|| anyhow::anyhow!("Chat not found"))?;
        let model = model?.ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        let messages_with_chunks = chat
            .find_related(message::Entity)
            .find_with_related(chunk::Entity)
            .all(db)
            .await?;

        let message = message::ActiveModel {
            chat_id: ActiveValue::Set(chat.id),
            kind: ActiveValue::Set(MessageKind::Assistant),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let mut publisher = ctx
            .channel
            .clone()
            .publish(chat_id)
            .context("only one publisher is allow at same time")?;

        let user_msg_id = messages_with_chunks
            .iter()
            .filter(|(m, _)| m.kind == MessageKind::User)
            .last()
            .context("no user message found")?
            .0
            .id;

        if publisher
            .publish(Token::Start {
                id: message.id,
                user_msg_id,
            })
            .is_err()
        {
            log::debug!("publisher was halted before completion start");
        }

        Ok(Self {
            model,
            chat: chat.into_active_model(),
            message: message.into_active_model(),
            messages_with_chunks,
            user,
            publisher,
            new_chunks: Vec::new(),
            ctx,
        })
    }

    pub fn set_mode(&mut self, mode: entity::ModeKind) {
        self.chat.mode = ActiveValue::Set(mode);
    }

    pub fn get_mode(&self) -> entity::ModeKind {
        self.chat.mode.clone().unwrap()
    }

    pub fn get_chat_id(&self) -> i32 {
        self.chat.id.clone().unwrap()
    }

    /// get user assistant id
    pub fn get_message_id(&self) -> i32 {
        self.message.id.clone().unwrap()
    }

    /// Adds a token to the completion context and publishes it to the channel.
    pub(super) fn add_token(&mut self, token: Token) -> Result<(), ()> {
        self.publisher.publish(token)
    }

    pub(super) fn add_token_force(&mut self, token: Token) {
        self.publisher.publish_force(token)
    }

    pub async fn put_stream<E>(
        &mut self,
        mut stream: impl Stream<Item = Result<Token, E>> + Unpin,
    ) -> Result<StreamEndReason, E> {
        while let Some(token) = stream.next().await {
            match token {
                Ok(token) => {
                    if self.add_token(token).is_err() {
                        return Ok(StreamEndReason::Halt);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(StreamEndReason::Exhausted)
    }

    pub fn update_usage(&mut self, price: f32, token_count: i32) {
        let new_price = self.message.price.take().unwrap_or(0.0) + price;
        self.message.price = ActiveValue::Set(new_price);

        let new_token = self.message.token_count.take().unwrap_or(0) + token_count;
        self.message.token_count = ActiveValue::Set(new_token);
    }

    /// Generates a title for the chat if it doesn't have one.
    async fn generate_title(&mut self) -> Result<(), anyhow::Error> {
        if !matches!(
            self.chat.title,
            ActiveValue::Set(None) | ActiveValue::NotSet | ActiveValue::Unchanged(None)
        ) {
            return Ok(());
        }
        let system_prompt = self.ctx.prompt.render(PromptKind::TitleGen, self)?;

        let mut message = vec![openrouter::Message::System(system_prompt)];

        message.extend(self.messages_with_chunks.iter().filter_map(|(m, chunks)| {
            // We ignore tool call or such, this is intentional.
            let text = chunks
                .iter()
                .filter_map(|c| match c.kind {
                    ChunkKind::Text => Some(c.content.as_str()),
                    ChunkKind::Report => Some(c.content.as_str()),
                    _ => None,
                })
                .collect::<String>();

            match m.kind {
                MessageKind::Hidden => None,
                MessageKind::User => Some(openrouter::Message::User(
                    text.chars().take(500).collect::<String>(),
                )),
                MessageKind::Assistant | MessageKind::DeepResearch => Some(
                    openrouter::Message::Assistant(text.chars().take(1000).collect::<String>()),
                ),
            }
        }));

        self.new_chunks.iter().for_each(|c| {
            if let ChunkKind::Text | ChunkKind::Report = c.kind.clone().unwrap() {
                if let Some(text) = c.content.try_as_ref() {
                    message.push(openrouter::Message::Assistant(
                        text.chars().take(1000).collect::<String>(),
                    ));
                }
            }
        });

        let model = self.model.get_config().context("invalid config")?;

        let completion = self.ctx.openrouter.complete(message, model.into()).await?;

        self.update_usage(completion.price as f32, completion.token as i32);

        static TRIMS: &[char] = &['\n', ' ', '\t', '`', '"', '\'', '*', '#'];

        let title = completion.response;

        let mut title = title
            .trim_matches(TRIMS)
            .chars()
            .take(60)
            .collect::<String>();

        if title.contains("\n") {
            title = title
                .split('\n')
                .next()
                .unwrap_or_default()
                .trim_matches(TRIMS)
                .chars()
                .collect::<String>();
        }

        self.chat.title = ActiveValue::set(Some(title.to_string()));

        self.publisher
            .publish_force(Token::Title(title.to_string()));

        Ok(())
    }

    pub fn add_error_chunk(&mut self, msg: String) {
        self.new_chunks
            .push(error_chunk(msg.clone(), self.message.id.clone().unwrap()));
        self.add_token_force(Token::Error(msg));
    }

    /// Saves the completion to the database.
    pub async fn save<E>(mut self, err: Option<E>) -> Result<(), anyhow::Error>
    where
        E: ToString,
    {
        let message_id = self.message.id.clone().unwrap();

        if let Some(err) = err {
            let err = err.to_string();
            self.add_error_chunk(err);
        }

        if let Err(err) = self.generate_title().await {
            self.add_error_chunk(err.to_string());
        }

        if self.new_chunks.is_empty() {
            self.add_error_chunk("No content generated, it's likely a bug of llumen.\nReport Here: https://github.com/pinkfuwa/llumen/issues/new".to_string());
        }

        let new_chunks = std::mem::take(&mut self.new_chunks);

        let new_chunks = new_chunks
            .into_iter()
            .map(|mut c| {
                c.message_id = ActiveValue::Set(message_id);
                c
            })
            .collect::<Vec<_>>();

        let db = &self.ctx.db;

        let chunks_affected = new_chunks.len();

        let last_insert_id = chunk::Entity::insert_many(new_chunks)
            .exec(db)
            .await?
            .last_insert_id;
        let chunk_ids =
            (last_insert_id - chunks_affected as i32 + 1..=last_insert_id).collect::<Vec<_>>();

        let token_count = self
            .message
            .token_count
            .clone()
            .try_as_ref()
            .copied()
            .unwrap_or(0);
        let cost = self
            .message
            .price
            .clone()
            .try_as_ref()
            .copied()
            .unwrap_or(0.0);

        log::trace!("publish complete token");
        self.publisher.publish_force(Token::Complete {
            message_id,
            chunk_ids,
            cost,
            token: token_count,
        });

        let db = &self.ctx.db;

        if let Err(err) = self.chat.update(db).await {
            if !matches!(err, DbErr::RecordNotUpdated) {
                return Err(err.into());
            }
        }
        if let Err(err) = self.message.update(db).await {
            if !matches!(err, DbErr::RecordNotUpdated) {
                return Err(err.into());
            }
        }

        Ok(())
    }

    pub fn latest_user_message(&self) -> Option<&str> {
        self.messages_with_chunks.last().and_then(|(m, chunks)| {
            if m.kind == MessageKind::User {
                let slice_vec: Vec<&str> = chunks
                    .iter()
                    .filter_map(|c| match c.kind {
                        ChunkKind::Text => Some(c.content.as_str()),
                        _ => None,
                    })
                    .collect();

                slice_vec.iter().copied().last()
            } else {
                None
            }
        })
    }
}
