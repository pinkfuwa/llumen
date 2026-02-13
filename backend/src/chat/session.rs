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
    /// reasoning details, citations).
    pub async fn apply_stream_result(&mut self, result: &openrouter::StreamResult) {
        if let Some(ref ann) = result.annotations {
            self.message.inner.add_annotation(ann.clone());
        }
        if let Some(ref rd) = result.reasoning_details {
            self.message.inner.add_reasoning_detail(rd.clone());
        }
        if !result.citations.is_empty() {
            self.message
                .inner
                .add_url_citation(result.citations.clone());
            self.publisher
                .publish(Token::UrlCitation(result.citations.clone()));
        }
        for img in &result.image {
            if let Some(id) = self.store_image(img).await {
                self.message.inner.add_image(id);
                self.publisher.publish(Token::Image(id));
            }
        }
    }

    async fn store_image(&self, img: &openrouter::Image) -> Option<i32> {
        // Image already has decoded data and mime_type
        let size = img.data.len();

        // Insert file record
        use ::entity::file;
        let file_record = file::ActiveModel {
            chat_id: Set(Some(self.chat.id)),
            owner_id: Set(None), // Generated images have no owner
            mime_type: Set(Some(img.mime_type.clone())),
            valid_until: Set(None), // Permanent
            ..Default::default()
        };

        let file_id = file::Entity::insert(file_record)
            .exec(&self.ctx.db)
            .await
            .ok()?
            .last_insert_id;

        // Store in BlobDB
        let byte_stream = tokio_stream::iter(vec![bytes::Bytes::from(img.data.clone())]);
        self.ctx
            .blob
            .insert(file_id, size, byte_stream)
            .await
            .ok()?;

        log::info!("store_image: stored image as file_id={}", file_id);
        Some(file_id)
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

        Ok(())
    }

    /// Generates and persists a chat title if one doesn't exist.
    /// Should be called after the completion finishes but before save().
    pub async fn try_generate_title(&mut self) -> Result<()> {
        if self.chat.title.is_some() {
            log::debug!("try_generate_title: chat already has title, skipping");
            return Ok(());
        }

        log::info!("try_generate_title: attempting to generate title");

        if let Some(title) = self.generate_title().await {
            log::info!("try_generate_title: generated title: '{}'", title);
            self.publisher.publish(Token::Title(title.clone()));

            // Create a partial ActiveModel with only id and title set
            let chat_active = chat::ActiveModel {
                id: Set(self.chat.id),
                title: Set(Some(title.clone())),
                ..Default::default()
            };
            chat::Entity::update(chat_active).exec(&self.ctx.db).await?;

            // Update the local copy so we don't try again
            self.chat.title = Some(title);
        } else {
            log::warn!("try_generate_title: generation returned None");
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

        // Extract text from assistant message chunks
        let assistant_text = if let protocol::MessageInner::Assistant(chunks) = &self.message.inner
        {
            chunks
                .iter()
                .filter_map(|chunk| match chunk {
                    protocol::AssistantChunk::Text(text) => Some(text.as_str()),
                    protocol::AssistantChunk::Reasoning(text) => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        };

        let assistant_truncated = assistant_text.chars().take(300).collect::<String>();

        let messages = vec![
            openrouter::Message::System(system),
            openrouter::Message::User(user_msg),
            openrouter::Message::Assistant {
                content: assistant_truncated,
                annotations: None,
                reasoning_details: None,
                images: Vec::new(),
            },
            openrouter::Message::User(
                "Please generate a concise title, starting with a emoji".to_string(),
            ),
        ];

        let model = self.openrouter_model();
        let option = openrouter::CompletionOption::builder()
            .max_reasoning_tokens(512)
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
