use std::sync::Arc;

use ::entity::*;
use anyhow::Context as _;
use sea_orm::*;
use tokio::join;
use tokio_stream::{Stream, StreamExt};

use super::tools::Tools;
use super::{
    channel::{self, Publisher},
    prompt::Prompt,
    token::Token,
};
use crate::chat::Strategies;
use crate::utils::model::ModelChecker;
use crate::{
    chat::prompt::PromptKind,
    openrouter::{self},
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
    pub(crate) db: DatabaseConnection,
    pub(crate) openrouter: openrouter::Openrouter,
    pub(crate) channel: Arc<channel::Context<Token>>,
    pub(crate) prompt: Arc<Prompt>,
    pub(crate) blob: Arc<BlobDB>,
    pub(crate) tools: Tools,
    pub strategies: Strategies,
}

impl Context {
    /// Creates a chat subsystem context that prepares routing, prompt, and tool dependencies.
    pub fn new(
        db: DatabaseConnection,
        openrouter: openrouter::Openrouter,
        blob: Arc<BlobDB>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            db,
            openrouter,
            channel: Arc::new(channel::Context::new()),
            prompt: Arc::new(Prompt::new()?),
            blob,
            tools: Tools::new(),
            strategies: Strategies::new(),
        })
    }

    /// Prepares a completion session for the specified user/chat/model tuple.
    pub fn get_completion_session(
        self: &Arc<Self>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> impl std::future::Future<Output = Result<CompletionSession, anyhow::Error>> + '_ {
        CompletionSession::new(self.clone(), user_id, chat_id, model_id)
    }
    /// Halts the streaming completion for the chat, notifying any subscribers.
    pub async fn halt_completion(&self, chat_id: i32) {
        self.channel.stop(chat_id).await
    }
    /// Subscribes to the chat token stream, optionally resuming from a cursor.
    pub fn subscribe(
        self: Arc<Self>,
        chat_id: i32,
        cursor: Option<channel::Cursor>,
    ) -> impl Stream<Item = Token> + 'static {
        self.channel.clone().subscribe(chat_id, cursor)
    }
    /// Returns true when no publisher currently owns the channel for the chat.
    pub fn is_streaming(&self, chat_id: i32) -> bool {
        !self.channel.publishable(chat_id)
    }

    /// Retrieves the list of model identifiers exposed by the OpenRouter client.
    pub async fn get_model_ids(&self) -> Vec<String> {
        self.openrouter.get_model_ids().await
    }

    /// Returns the capability information for a single OpenRouter model.
    pub async fn get_capability(&self, model: &openrouter::Model) -> openrouter::Capability {
        self.openrouter.get_capability(model).await
    }

    /// Delegates completion processing to the configured strategy.
    pub fn process(
        self: Arc<Self>,
        session: CompletionSession,
    ) -> futures_util::future::BoxFuture<'static, anyhow::Result<()>> {
        self.strategies.process(self.clone(), session)
    }
}

/// Per-request session for a completion.
///
/// Represents one HTTP request to create an assistant message.
/// Holds DB entities, SSE publisher, and reference to global services.
///
/// Lifetime: Created at request start, dropped after DB save.
pub struct CompletionSession {
    /// The model used for the completion.
    pub(crate) model: model::Model,
    /// The chat the completion belongs to.
    pub(crate) chat: chat::ActiveModel,
    /// The message the completion belongs to.
    pub(crate) message: message::Model,
    /// The previous chunks in the chat.
    pub(crate) messages: Vec<message::Model>,
    /// The user who initiated the completion.
    pub(crate) user: user::Model,
    /// The publisher for the completion's tokens.
    publisher: Publisher<Token>,
    /// The global context.
    ctx: Arc<Context>,
}

impl CompletionSession {
    /// Creates a new completion session.
    /// Loads user, chat, model, and drafting message records from the database for initialization.
    async fn load_entities(
        ctx: &Context,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> Result<
        (
            user::Model,
            (chat::Model, Vec<message::Model>),
            model::Model,
            message::Model,
        ),
        anyhow::Error,
    > {
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
        let chat_with_msgs = chat_with_msgs?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Chat not found"))?;
        let model = model?.ok_or_else(|| anyhow::anyhow!("Model not found"))?;

        Ok((user, chat_with_msgs, model, msg))
    }

    /// Converts a persistent chat record into the mutable active model used during completion.
    fn make_chat_active_model(chat: chat::Model, model_id: i32) -> chat::ActiveModel {
        let mut chat = chat.into_active_model();
        chat.model_id = ActiveValue::Set(Some(model_id));
        chat
    }

    /// Claims the exclusive publisher slot for the specified chat.
    fn claim_publisher(
        ctx: &Arc<Context>,
        chat_id: i32,
    ) -> Result<Publisher<Token>, anyhow::Error> {
        ctx.channel
            .clone()
            .publish(chat_id)
            .context("only one publisher is allow at same time")
    }

    /// Emits the initial start token that links the completion to the preceding user message.
    fn init_publisher_tokens(
        mut publisher: Publisher<Token>,
        msgs: &[message::Model],
        msg: &message::Model,
    ) -> Result<Publisher<Token>, anyhow::Error> {
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

        Ok(publisher)
    }

    /// Creates and prepares the completion context for a new request.
    pub async fn new(
        ctx: Arc<Context>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
    ) -> Result<Self, anyhow::Error> {
        let (user, (chat, msgs), model, msg) =
            Self::load_entities(&ctx, user_id, chat_id, model_id).await?;

        let chat = Self::make_chat_active_model(chat, model.id);
        let publisher = Self::claim_publisher(&ctx, chat_id)?;
        let publisher = Self::init_publisher_tokens(publisher, &msgs, &msg)?;

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

    /// Updates the chat mode stored for this completion.
    pub fn set_mode(&mut self, mode: ModeKind) {
        self.chat.mode = ActiveValue::Set(mode);
    }

    /// Returns the stored chat mode.
    pub fn get_mode(&self) -> ModeKind {
        self.chat.mode.clone().unwrap()
    }

    /// Returns the active chat identifier.
    pub fn get_chat_id(&self) -> i32 {
        self.chat.id.clone().unwrap()
    }

    /// Returns the requesting user's identifier.
    pub fn get_user_id(&self) -> i32 {
        self.user.id
    }

    /// Returns the assistant message identifier in progress.
    pub fn get_message_id(&self) -> i32 {
        self.message.id
    }

    /// Creates a StreamWriter for this session.
    ///
    /// The StreamWriter provides a cleaner interface for writing tokens
    /// compared to directly using the publisher.
    pub(crate) fn create_stream_writer(&mut self) -> crate::chat::StreamWriter<'_> {
        crate::chat::StreamWriter::new(&mut self.publisher)
    }

    /// Adds a token to the completion context and publishes it to the channel.
    pub(crate) fn add_token(&mut self, token: Token) {
        self.publisher.publish(token)
    }

    /// Streams tokens from the provided source into the publisher, respecting halt signals.
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

    /// Records usage metrics arising from the completion.
    /// Records accumulated price and token usage.
    pub fn update_usage(&mut self, price: f32, token_count: i32) {
        self.message.price += price;
        self.message.token_count += token_count;
    }

    /// Parses the stored model configuration for use with OpenRouter.
    pub fn get_model_config(&self) -> anyhow::Result<ModelConfig> {
        <ModelConfig as ModelChecker>::from_toml(&self.model.config).context("invalid config")
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

        let model = self.get_model_config()?;

        let option = openrouter::CompletionOption::builder()
            .max_reasoning_tokens(512)
            .temperature(0.2)
            .build();

        let mut title = match self
            .ctx
            .openrouter
            .complete(messages, model.into(), option)
            .await
        {
            Ok(completion) => {
                self.update_usage(completion.price as f32, completion.token as i32);
                completion.response
            }
            Err(openrouter::Error::TextOutputNotSupported) => {
                // Model doesn't support text output, use last user message as fallback
                String::new()
            }
            Err(e) => return Err(e.into()),
        };

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

    /// Records a completion error, stores it on the message, and emits an error token.
    pub fn add_error(&mut self, msg: String) {
        self.message.inner.add_error(msg.clone());
        self.add_token(Token::Error(msg));
    }

    /// Saves the completion to the database and publishes the completion token.
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

    /// Returns the most recent user message text, if available.
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

/// TokenSink implementation for CompletionSession.
///
/// Delegates to existing methods (add_token, put_stream, update_usage).
/// This allows DeepAgent to stream tokens without tight coupling.
impl crate::chat::TokenSink for CompletionSession {
    fn add_token(&mut self, token: Token) {
        CompletionSession::add_token(self, token)
    }

    async fn put_stream<E: std::error::Error + Send + 'static>(
        &mut self,
        stream: impl Stream<Item = std::result::Result<Token, E>> + Unpin + Send,
    ) -> anyhow::Result<StreamEndReason> {
        CompletionSession::put_stream(self, stream)
            .await
            .map_err(|e| anyhow::anyhow!("Stream error: {}", e))
    }

    fn update_usage(&mut self, cost: f32, tokens: i32) {
        CompletionSession::update_usage(self, cost, tokens)
    }
}
