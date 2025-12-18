use std::sync::Arc;

use ::entity::*;
use anyhow::Context as _;
use sea_orm::*;
use tokio::join;
use tokio_stream::{Stream, StreamExt};

use super::tools::{CrawlTool, LuaReplTool, WebSearchTool};
use super::{
    channel::{self, Publisher},
    prompt::Prompt,
    token::Token,
};
use crate::chat::Configurations;
use crate::chat::deep_prompt::DeepPrompt;
use crate::utils::model::ModelChecker;
use crate::{
    chat::prompt::PromptKind,
    openrouter::{self, ReasoningEffort},
    utils::blob::BlobDB,
};
use protocol::*;

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
    pub configurations: Configurations,
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
            configurations: Configurations::new(),
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
    pub async fn halt_completion(&self, chat_id: i32) {
        self.channel.stop(chat_id).await
    }
    pub fn subscribe(
        self: Arc<Self>,
        chat_id: i32,
        cursor: Option<channel::Cursor>,
    ) -> impl Stream<Item = Token> + 'static {
        self.channel.clone().subscribe(chat_id, cursor)
    }
    pub fn is_streaming(&self, chat_id: i32) -> bool {
        !self.channel.publishable(chat_id)
    }

    pub fn get_model_ids(&self) -> Vec<String> {
        self.openrouter.get_model_ids()
    }

    pub fn get_capability(&self, model: &openrouter::Model) -> openrouter::Capability {
        self.openrouter.get_capability(model)
    }

    pub fn process(
        self: Arc<Self>,
        completion_ctx: CompletionContext,
    ) -> futures_util::future::BoxFuture<'static, anyhow::Result<()>> {
        self.configurations.process(self.clone(), completion_ctx)
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
    pub(super) message: message::Model,
    /// The previous chunks in the chat.
    pub(super) messages: Vec<message::Model>,
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

        let (user, chat_with_msgs, model, msg) = join!(
            user::Entity::find_by_id(user_id).one(db),
            chat::Entity::find_by_id(chat_id)
                .find_with_related(message::Entity)
                .all(db),
            model::Entity::find_by_id(model_id).one(db),
            message::ActiveModel {
                chat_id: ActiveValue::Set(chat_id),
                inner: ActiveValue::Set(MessageInner::default()),
                ..Default::default()
            }
            .insert(db)
        );

        let msg = msg?;
        let user = user?.ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let (chat, msgs) = chat_with_msgs?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;
        let model = model?.ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        let mut chat = chat.into_active_model();
        chat.model_id = ActiveValue::Set(Some(model.id));

        let mut publisher = ctx
            .channel
            .clone()
            .publish(chat_id)
            .context("only one publisher is allow at same time")?;

        let user_msg_id = msgs
            .iter()
            .filter(|m| matches!(m.inner, MessageInner::User { .. }))
            .last()
            .context("no user message found")?
            .id;

        publisher.publish(Token::Start {
            id: msg.id,
            user_msg_id,
        });

        Ok(Self {
            model,
            chat,
            message: msg,
            messages: msgs,
            user,
            publisher,
            ctx,
        })
    }

    pub fn set_mode(&mut self, mode: ModeKind) {
        self.chat.mode = ActiveValue::Set(mode);
    }

    pub fn get_mode(&self) -> ModeKind {
        self.chat.mode.clone().unwrap()
    }

    pub fn get_chat_id(&self) -> i32 {
        self.chat.id.clone().unwrap()
    }

    pub fn get_user_id(&self) -> i32 {
        self.user.id
    }

    /// get user assistant id
    pub fn get_message_id(&self) -> i32 {
        self.message.id
    }

    /// Adds a token to the completion context and publishes it to the channel.
    pub(super) fn add_token(&mut self, token: Token) {
        self.publisher.publish(token)
    }

    pub async fn put_stream<E>(
        &mut self,
        mut stream: impl Stream<Item = Result<Token, E>> + Unpin,
    ) -> Result<StreamEndReason, E> {
        let halt_fut = self.publisher.wait_halt();
        let fut = async {
            while let Some(token) = stream.next().await {
                match token {
                    Ok(token) => self.add_token(token),
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        };
        tokio::select! {
            _ = halt_fut => {
                Ok(StreamEndReason::Halt)
            },
            result = fut => {
                result.map(|_|StreamEndReason::Exhausted)
            }
        }
    }

    pub fn update_usage(&mut self, price: f32, token_count: i32) {
        self.message.price += price;
        self.message.token_count += token_count;
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

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        if let Some(user_msg) = self.messages.iter().find_map(|m| match &m.inner {
            MessageInner::User { text, .. } => Some(text.to_string()),
            MessageInner::Assistant(_) => None,
        }) {
            messages.push(openrouter::Message::User(user_msg));
        }

        if let Some(assistant_chunks) =
            self.message
                .inner
                .as_assistant()
                .unwrap()
                .iter()
                .find_map(|m| match m {
                    AssistantChunk::Text(x) => Some(x),
                    _ => None,
                })
        {
            let text = assistant_chunks.chars().take(300).collect::<String>();
            messages.push(openrouter::Message::Assistant {
                content: text,
                annotations: None,
                reasoning_details: None,
                images: Vec::new(),
            });
        }

        messages.push(openrouter::Message::User(
            "Please generate a concise title, starting with a emoji".to_string(),
        ));

        let model = <ModelConfig as ModelChecker>::from_toml(&self.model.config)
            .context("invalid config")?;

        let option = openrouter::CompletionOption::builder()
            .reasoning_effort(ReasoningEffort::Low)
            .max_tokens(512)
            .temperature(0.2)
            .build();

        let completion = self
            .ctx
            .openrouter
            .complete(messages, model.into(), option)
            .await?;

        self.update_usage(completion.price as f32, completion.token as i32);

        let mut title = completion.response;

        if title.is_empty() {
            let last_user_message = self.messages.iter().find_map(|msg| match &msg.inner {
                MessageInner::User { text, .. } => Some(text.clone()),
                _ => None,
            });

            if let Some(last_user_message) = last_user_message {
                title = last_user_message;
            }
        }

        static TRIMS: &[char] = &['\n', ' ', '\t', '`', '"', '\'', '*', '#'];

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

        self.add_token(Token::Title(title.to_string()));

        Ok(())
    }

    pub fn add_error(&mut self, msg: String) {
        self.message.inner.add_error(msg.clone());
        self.add_token(Token::Error(msg));
    }

    /// Saves the completion to the database.
    pub async fn save(mut self) -> Result<(), anyhow::Error> {
        let message_id = self.message.id;

        if let Err(err) = self.generate_title().await {
            self.add_error(err.to_string());
        }

        if self.message.inner.is_empty() {
            self.add_error("No content generated, it's likely a bug of llumen.\nReport Here: https://github.com/pinkfuwa/llumen/issues/new".to_string());
        }

        let token_count = self.message.token_count;
        let cost = self.message.price;

        log::trace!("publish complete token");
        self.add_token(Token::Complete {
            message_id,
            cost,
            token: token_count,
        });

        let db = &self.ctx.db;

        if let Err(err) = self.chat.update(db).await {
            if !matches!(err, DbErr::RecordNotUpdated) {
                return Err(err.into());
            }
        }
        let mut msg = self.message.into_active_model();
        msg.full_change();
        if let Err(err) = msg.update(db).await {
            if !matches!(err, DbErr::RecordNotUpdated) {
                return Err(err.into());
            }
        }

        Ok(())
    }

    pub fn latest_user_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .filter_map(|m| {
                if let MessageInner::User { text, files: _ } = &m.inner {
                    return Some(text.as_str());
                } else {
                    None
                }
            })
            .last()
    }
}
