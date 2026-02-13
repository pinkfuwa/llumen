//! CompletionSession – mutable state for a single completion run.
//!
//! A session is created per chat-message request.  It loads the model
//! config, previous messages, and user preferences, then exposes
//! helpers for message assembly, streaming, and persistence.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use ::entity::{chat, message, prelude::*, user};
use ::entity::model as entity_model;
use protocol::*;
use sea_orm::*;

use super::context::{Context, StreamEndReason};
use super::converter;
use super::token::Token;
use crate::openrouter;
use crate::utils::model::ModelChecker;

/// Loaded information about the model driving this session.
pub struct SessionModel {
    pub id: i32,
    pub config: ModelConfig,
}

/// Loaded user info needed during completion.
pub struct SessionUser {
    pub id: i32,
    pub preference: UserPreference,
}

/// A single completion run.  Holds mutable message state, a reference
/// back to the shared [`Context`], and a streaming publisher.
pub struct CompletionSession {
    ctx: Arc<Context>,
    pub user: SessionUser,
    pub model: SessionModel,
    pub chat: chat::Model,
    pub message: message::Model,
    pub(super) history: Vec<message::Model>,
    cost: f32,
    token_count: i32,
    publisher: super::channel::Publisher<Token>,
}

impl CompletionSession {
    /// Loads user, chat, model, and history from the database.
    pub async fn new(ctx: Arc<Context>, user_id: i32, chat_id: i32, model_id: i32) -> Result<Self> {
        let db = &ctx.db;

        let (user, chat, model_entity) = tokio::try_join!(
            async {
                user::Entity::find_by_id(user_id)
                    .one(db)
                    .await?
                    .context("user not found")
            },
            async {
                chat::Entity::find_by_id(chat_id)
                    .one(db)
                    .await?
                    .context("chat not found")
            },
            async {
                entity_model::Entity::find_by_id(model_id)
                    .one(db)
                    .await?
                    .context("model not found")
            },
        )?;

        let model_config = <ModelConfig as ModelChecker>::from_toml(&model_entity.config)?;

        let history = Message::find()
            .filter(message::Column::ChatId.eq(chat_id))
            .order_by_asc(message::Column::Id)
            .all(db)
            .await?;

        // Create a placeholder assistant message that strategies will populate.
        let new_msg = message::ActiveModel {
            chat_id: Set(chat_id),
            price: Set(0.0),
            token_count: Set(0),
            inner: Set(MessageInner::default()),
            ..Default::default()
        };
        let insert_result = message::Entity::insert(new_msg).exec(db).await?;
        let msg_id = insert_result.last_insert_id;

        let message = message::Model {
            id: msg_id,
            chat_id,
            price: 0.0,
            token_count: 0,
            inner: MessageInner::default(),
        };

        let publisher = ctx
            .channel
            .clone()
            .publish(chat_id)
            .context("another session is already streaming on this chat")?;

        Ok(Self {
            ctx,
            user: SessionUser {
                id: user_id,
                preference: user.preference,
            },
            model: SessionModel {
                id: model_id,
                config: model_config,
            },
            chat,
            message,
            history,
            cost: 0.0,
            token_count: 0,
            publisher,
        })
    }

    // ------------------------------------------------------------------
    // Message assembly  [system_prompt, previous_messages, context, user_query]
    // ------------------------------------------------------------------

    /// Builds the full OpenRouter message array for the current session.
    pub fn assemble_messages(
        &self,
        ctx: &Context,
        _option: openrouter::CompletionOption,
    ) -> Result<Vec<openrouter::Message>> {
        let locale = self.locale();
        let mode = self.chat.mode;

        // 1. System prompt
        let system_prompt = match mode {
            ModeKind::Normal => {
                ctx.prompt
                    .render_normal(locale, &self.model.config.display_name, "")?
            }
            ModeKind::Search => {
                ctx.prompt
                    .render_search(locale, &self.model.config.display_name, "")?
            }
            ModeKind::Research => ctx.prompt.render_coordinator(locale)?,
        };

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        // 2. Previous messages (from DB → openrouter format)
        let history_msgs = converter::history_to_openrouter(&self.history, &ctx.blob);
        messages.extend(history_msgs);

        // 3. Context injection
        let is_llumen_related = self
            .latest_user_message()
            .map(|m| m.to_lowercase().contains("llumen"))
            .unwrap_or(false);
        let time_str = time::OffsetDateTime::now_utc()
            .format(super::prompt::TIME_FORMAT)
            .unwrap_or_default();
        let context_prompt =
            ctx.prompt
                .render_context(is_llumen_related, &time_str, self.chat.title.as_deref())?;
        if !context_prompt.trim().is_empty() {
            messages.push(openrouter::Message::System(context_prompt));
        }

        Ok(messages)
    }

    // Helpers
    pub fn locale(&self) -> &str {
        self.user.preference.locale.as_deref().unwrap_or("en-US")
    }

    pub fn latest_user_message(&self) -> Option<&str> {
        self.history.iter().rev().find_map(|m| match &m.inner {
            MessageInner::User { text, .. } => Some(text.as_str()),
            _ => None,
        })
    }

    /// Builds an `openrouter::Model` from the stored config.
    pub fn openrouter_model(&self) -> openrouter::Model {
        self.model.config.clone().into()
    }

    // Streaming (TokenSink implementation)
    pub fn add_token(&mut self, token: Token) {
        self.publisher.publish(token);
    }

    pub fn add_error(&mut self, msg: String) {
        self.publisher.publish(Token::Error(msg.clone()));
        self.message.inner.add_error(msg);
    }

    pub fn update_usage(&mut self, cost: f32, tokens: i32) {
        self.cost += cost;
        self.token_count += tokens;
    }

    /// Drains a mapped OpenRouter token stream, publishing each token
    /// and returning `Halt` if a stop was requested.
    pub async fn put_stream<S>(&mut self, stream: S) -> Result<StreamEndReason>
    where
        S: tokio_stream::Stream<Item = Result<Token, openrouter::Error>> + Unpin,
    {
        use tokio_stream::StreamExt;
        tokio::pin!(stream);

        loop {
            tokio::select! {
                biased;
                _ = self.publisher.wait_halt() => {
                    return Ok(StreamEndReason::Halt);
                }
                item = StreamExt::next(&mut stream) => {
                    match item {
                        Some(Ok(token)) => self.publisher.publish(token),
                        Some(Err(e)) => {
                            log::error!("stream error: {e}");
                            self.publisher.publish(Token::Error(e.to_string()));
                            return Ok(StreamEndReason::Exhausted);
                        }
                        None => return Ok(StreamEndReason::Exhausted),
                    }
                }
            }
        }
    }

    /// Applies metadata from a finished stream result (annotations, images,
    /// reasoning details).
    pub fn apply_stream_result(&mut self, result: &openrouter::StreamResult) {
        if let Some(ref ann) = result.annotations {
            self.message.inner.add_annotation(ann.clone());
        }
        if let Some(ref rd) = result.reasoning_details {
            self.message.inner.add_reasoning_detail(rd.clone());
        }
        for img in &result.image {
            let file_id = self.store_image(img);
            if let Some(id) = file_id {
                self.message.inner.add_image(id);
                self.publisher.publish(Token::Image(id));
            }
        }
    }

    fn store_image(&self, _img: &openrouter::Image) -> Option<i32> {
        // TODO: store image to blob DB and return file id
        None
    }

    /// Appends assistant chunks to the message being built.
    pub fn extend_chunks(&mut self, chunks: Vec<AssistantChunk>) {
        if let Some(existing) = self.message.inner.as_assistant() {
            existing.extend(chunks);
        }
    }

    pub fn add_chunk(&mut self, chunk: AssistantChunk) {
        if let Some(existing) = self.message.inner.as_assistant() {
            existing.push(chunk);
        }
    }

    // Persistence

    /// Saves the completed message to the database and emits the Complete
    /// token.
    pub async fn save(mut self) -> Result<()> {
        let db = &self.ctx.db;

        // Persist the assistant message
        let mut active: message::ActiveModel = self.message.clone().into();
        active.price = Set(self.cost);
        active.token_count = Set(self.token_count);
        active.inner = Set(self.message.inner.clone());
        message::Entity::update(active).exec(db).await?;

        // Emit Complete token
        self.publisher.publish(Token::Complete {
            message_id: self.message.id,
            cost: self.cost,
            token: self.token_count,
        });

        // Generate title if missing
        if self.chat.title.is_none() {
            log::info!("save: chat title is None, attempting to generate");
            if let Some(title) = self.generate_title().await {
                log::info!("save: generated title: '{}'", title);
                self.publisher.publish(Token::Title(title.clone()));
                let mut chat_active: chat::ActiveModel = self.chat.clone().into();
                chat_active.title = Set(Some(title));
                chat::Entity::update(chat_active).exec(db).await?;
            } else {
                log::warn!("save: title generation returned None");
            }
        } else {
            log::debug!("save: chat already has title: {:?}", self.chat.title);
        }

        Ok(())
    }

    async fn generate_title(&self) -> Option<String> {
        let locale = self.locale();
        let system = self.ctx.prompt.render_title_generation(locale).ok()?;

        let user_msg = match self.latest_user_message() {
            Some(msg) => msg.to_string(),
            None => {
                log::warn!(
                    "generate_title: no user message found in history (history len: {})",
                    self.history.len()
                );
                return None;
            }
        };

        let messages = vec![
            openrouter::Message::System(system),
            openrouter::Message::User(user_msg),
        ];

        let model = self.openrouter_model();
        let option = openrouter::CompletionOption::builder()
            .max_tokens(50)
            .build();

        let result = self
            .ctx
            .openrouter
            .complete(messages, model, option)
            .await
            .ok()?;
        let title = result.response.trim().to_string();

        log::info!("generate_title: got response: '{}'", title);

        if title.is_empty() { None } else { Some(title) }
    }
}

/// Trait so that strategies and DeepAgent can write tokens without knowing
/// the concrete session type.
pub trait TokenSink {
    fn add_token(&mut self, token: Token);
    fn add_error(&mut self, msg: String);
    fn update_usage(&mut self, cost: f32, tokens: i32);
    fn put_stream(
        &mut self,
        stream: impl tokio_stream::Stream<Item = Result<Token, openrouter::Error>> + Unpin + Send,
    ) -> impl std::future::Future<Output = Result<StreamEndReason>> + Send;
}

impl TokenSink for CompletionSession {
    fn add_token(&mut self, token: Token) {
        self.add_token(token);
    }
    fn add_error(&mut self, msg: String) {
        self.add_error(msg);
    }
    fn update_usage(&mut self, cost: f32, tokens: i32) {
        self.update_usage(cost, tokens);
    }
    async fn put_stream(
        &mut self,
        stream: impl tokio_stream::Stream<Item = Result<Token, openrouter::Error>> + Unpin + Send,
    ) -> Result<StreamEndReason> {
        self.put_stream(stream).await
    }
}
