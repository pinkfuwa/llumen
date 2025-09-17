use std::sync::Arc;

use anyhow::Context as _;
use entity::{ChunkKind, MessageKind, chat, chunk, message, model, user};
use futures_util::{Stream, StreamExt};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    ModelTrait, QueryFilter,
};

use super::{
    channel::{self, Publisher},
    prompt::Prompt,
    token::Token,
};
use crate::{chat::prompt::PromptKind, openrouter};

#[derive(Debug, Clone, Copy)]
pub enum StreamEndReason {
    Halt,
    Exhausted,
}

/// The global context for the chat system.
/// It holds the database connection, the OpenRouter client, and the channel context.
pub struct PipelineContext {
    pub(super) db: DatabaseConnection,
    pub(super) openrouter: openrouter::Openrouter,
    pub(super) channel: channel::Context<Token>,
    pub(super) prompt: Prompt,
}

impl PipelineContext {
    // TODO: put API Key in main
    pub fn new(db: DatabaseConnection) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db,
            openrouter: openrouter::Openrouter::new(),
            channel: channel::Context::new(),
            prompt: Prompt::new(),
        })
    }

    pub fn get_completion_context(
        self: &Arc<Self>,
        user_id: i32,
        chat_id: i32,
    ) -> impl std::future::Future<Output = Result<CompletionContext, anyhow::Error>> + '_ {
        CompletionContext::new(self.clone(), user_id, chat_id)
    }
    pub fn halt_completion(&self, chat_id: i32) {
        self.channel.stop(chat_id)
    }
    pub fn subscribe(self: Arc<Self>, chat_id: i32) -> impl Stream<Item = Token> {
        self.channel.subscribe(chat_id).flatten()
    }
}

/// Creates a new error chunk with the given message.
fn error_chunk(msg: String) -> chunk::ActiveModel {
    chunk::ActiveModel {
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
    ctx: Arc<PipelineContext>,
}

impl CompletionContext {
    /// Creates a new completion context.
    pub async fn new(
        ctx: Arc<PipelineContext>,
        user_id: i32,
        chat_id: i32,
    ) -> Result<Self, anyhow::Error> {
        let db = &ctx.db;
        let user = user::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let (chat, model) = chat::Entity::find_by_id(chat_id)
            .filter(chat::Column::OwnerId.eq(user_id))
            .find_also_related(model::Entity)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;

        let model = model.ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        let messages_with_chunks = chat
            .find_related(message::Entity)
            .find_with_related(chunk::Entity)
            .all(db)
            .await?;

        let message = message::ActiveModel {
            chat_id: ActiveValue::Set(chat.id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(Self {
            model,
            chat: chat.into_active_model(),
            message: message.into_active_model(),
            messages_with_chunks,
            user,
            publisher: ctx.channel.publish(chat_id),
            new_chunks: Vec::new(),
            ctx,
        })
    }

    pub fn get_chat_id(&self) -> i32 {
        self.chat.id.clone().unwrap()
    }

    pub fn get_message_id(&self) -> i32 {
        self.message.id.clone().unwrap()
    }

    /// Adds a token to the completion context and publishes it to the channel.
    pub fn add_token(&mut self, token: Token) -> Result<(), ()> {
        self.publisher.publish(token)
    }

    pub fn add_token_force(&mut self, token: Token) {
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
                Err(e) => {
                    return Err(e);
                }
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
        if self.chat.title.is_set() {
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
                MessageKind::User => Some(openrouter::Message::User(text)),
                MessageKind::Assistant | MessageKind::DeepResearch => {
                    Some(openrouter::Message::Assistant(text))
                }
            }
        }));

        let model = self.model.get_config().context("invalid config")?;

        let completion = self.ctx.openrouter.complete(message, model.into()).await?;

        self.update_usage(completion.price as f32, completion.token as i32);

        // TODO: trim the title from both ends.
        // static TRIMS: &[char] = &['\n', ' ', '\t', '`', '"', '\''];

        self.chat.title = ActiveValue::set(Some(completion.response));

        Ok(())
    }

    /// Saves the completion to the database.
    pub async fn save(mut self, err: Option<String>) -> Result<(), anyhow::Error> {
        let mut chunks = Vec::new();

        if let Some(err) = err {
            chunks.push(error_chunk(err));
        }
        if let Err(e) = self.generate_title().await {
            chunks.push(error_chunk(format!("Failed to generate title: {}", e)));
        }

        let db = &self.ctx.db;

        let message_id = self.message.id.clone().unwrap();

        self.chat.update(db).await?;
        self.message.update(db).await?;

        let chunks_affected = self.new_chunks.len();
        let last_insert_id = chunk::Entity::insert_many(self.new_chunks)
            .exec(db)
            .await?
            .last_insert_id;
        let chunk_ids =
            (last_insert_id - chunks_affected as i32 + 1..=last_insert_id).collect::<Vec<_>>();

        self.publisher.publish_force(Token::Complete {
            message_id,
            chunk_ids,
        });

        Ok(())
    }
}
