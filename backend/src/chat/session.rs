//! CompletionSession – mutable state for a single completion run.
//!
//! A session is created per chat-message request.  It loads the model
//! config, previous messages, and user preferences, then exposes
//! helpers for message assembly, streaming, and persistence.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use ::entity::{chat, message, prelude::*, user};
use ::entity::file;
use ::entity::model as entity_model;
use futures_util::TryStreamExt;
use protocol::*;
use sea_orm::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml::de;

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
    file_mime_types: Vec<(i32, Option<String>)>,
    cost: f32,
    token_count: i32,
    publisher: super::channel::Publisher<Token>,
    mode: protocol::ModeKind,
}

impl CompletionSession {
    fn collect_history_file_ids(history: &[message::Model]) -> Vec<i32> {
        let mut file_ids = Vec::new();

        for record in history {
            match &record.inner {
                MessageInner::User { files, .. } => {
                    file_ids.extend(files.iter().map(|file| file.id));
                }
                MessageInner::Assistant(chunks) => {
                    for chunk in chunks {
                        if let AssistantChunk::ToolResult { files, .. } = chunk {
                            file_ids.extend(files.iter().map(|file| file.id));
                        }
                    }
                }
            }
        }

        file_ids.sort_unstable();
        file_ids.dedup();
        file_ids
    }

    async fn load_history_file_mime_types(
        db: &DatabaseConnection,
        history: &[message::Model],
    ) -> Result<Vec<(i32, Option<String>)>> {
        let file_ids = Self::collect_history_file_ids(history);
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let records = file::Entity::find()
            .select_only()
            .column(file::Column::Id)
            .column(file::Column::MimeType)
            .filter(file::Column::Id.is_in(file_ids))
            .into_tuple::<(i32, Option<String>)>()
            .all(db)
            .await?;

        Ok(records)
    }

    fn set_file_mime_type(&mut self, file_id: i32, mime_type: Option<String>) {
        if let Some((_, existing_mime_type)) = self
            .file_mime_types
            .iter_mut()
            .find(|(existing_id, _)| *existing_id == file_id)
        {
            *existing_mime_type = mime_type;
            return;
        }

        self.file_mime_types.push((file_id, mime_type));
    }

    pub fn file_mime_type(&self, file_id: i32) -> Option<&str> {
        self.file_mime_types
            .iter()
            .find(|(existing_id, _)| *existing_id == file_id)
            .and_then(|(_, mime_type)| mime_type.as_deref())
    }

    /// Loads user, chat, model, and history from the database.
    pub async fn new(
        ctx: Arc<Context>,
        user_id: i32,
        chat_id: i32,
        model_id: i32,
        mode: protocol::ModeKind,
    ) -> Result<Self> {
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

        let publisher = ctx
            .channel
            .clone()
            .publish(chat_id)
            .context("another session is already streaming on this chat")?;

        let history = Message::find()
            .filter(message::Column::ChatId.eq(chat_id))
            .order_by_asc(message::Column::Id)
            .all(db)
            .await?;

        let file_mime_types = Self::load_history_file_mime_types(db, &history).await?;

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

        log::debug!(
            "session created: chat_id={}, user_id={}, model_id={}, msg_id={}",
            chat_id,
            user_id,
            model_id,
            msg_id
        );

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
            file_mime_types,
            cost: 0.0,
            token_count: 0,
            publisher,
            mode,
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
        let model_id = self.model.config.model_id.as_str();
        let model_supported_parameters = self.model_supported_parameters();
        let (image_model_id, video_model_id) = self.media_model_ids();
        let (image_model_supported_parameters, video_model_supported_parameters) =
            self.media_model_prompt_parameters();

        // 1. System prompt
        let system_prompt = match mode {
            ModeKind::Normal => ctx.prompt.render_normal(
                locale,
                &self.model.config.display_name,
                model_id,
                "",
                &model_supported_parameters,
            )?,
            ModeKind::Search => ctx.prompt.render_search(
                locale,
                &self.model.config.display_name,
                model_id,
                "",
                &model_supported_parameters,
            )?,
            ModeKind::Media => ctx.prompt.render_media(
                locale,
                &self.model.config.display_name,
                model_id,
                "",
                &model_supported_parameters,
                image_model_id,
                video_model_id,
                &image_model_supported_parameters,
                &video_model_supported_parameters,
            )?,
            #[cfg(feature = "deep-research")]
            ModeKind::Research => ctx.prompt.render_coordinator(locale)?,
            #[cfg(not(feature = "deep-research"))]
            ModeKind::Research => ctx.prompt.render_normal(
                locale,
                &self.model.config.display_name,
                model_id,
                "",
                &model_supported_parameters,
            )?,
        };

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        // 2. Previous messages (from DB → openrouter format)
        let history_msgs =
            converter::history_to_openrouter(&self.history, &ctx.blob, &self.file_mime_types);
        messages.extend(history_msgs);

        // 3. Context injection as USER message BEFORE the last user query
        // This placement ensures:
        // - Image-only models get context (they take last message)
        // - Gemini compatibility (single system prompt preserved)
        // - Prompt caching is not ruined (system prompt stays clean)
        let is_llumen_related = self
            .latest_user_message()
            .map(|m| m.to_lowercase().contains("llumen"))
            .unwrap_or(false);
        let time_str = time::OffsetDateTime::now_local()
            .map(|dt| dt.format(super::prompt::TIME_FORMAT))
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc().format(super::prompt::TIME_FORMAT))
            .unwrap_or_default();
        let contain_dollar_sign = self.assert_assistant_message(|t| t.contains("$"));
        let context_prompt = ctx.prompt.render_context(
            is_llumen_related,
            &time_str,
            self.chat.title.as_deref(),
            contain_dollar_sign,
        )?;

        if !context_prompt.trim().is_empty() {
            // Find the position of the last user message and insert context before it
            let last_user_pos = messages.iter().enumerate().rev().find_map(|(i, msg)| {
                matches!(
                    msg,
                    openrouter::Message::User(_) | openrouter::Message::MultipartUser { .. }
                )
                .then_some(i)
            });

            if let Some(pos) = last_user_pos {
                // Insert context before the last user message
                messages.insert(pos, openrouter::Message::User(context_prompt));
            } else {
                // No user message found, append context at the end
                messages.push(openrouter::Message::User(context_prompt));
            }
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

    pub fn assert_assistant_message(&self, f: impl Fn(&str) -> bool) -> bool {
        self.history.iter().any(|m| match &m.inner {
            MessageInner::Assistant(inner) => {
                inner.iter().filter_map(|c| c.as_text()).any(|t| f(&t))
            }
            _ => false,
        })
    }

    fn model_supported_parameters(&self) -> Vec<String> {
        let mut parameters = Vec::new();
        let parameter = &self.model.config.parameter;
        let capability = &self.model.config.capability;

        if parameter.temperature.is_some() {
            parameters.push("temperature".to_string());
        }
        if parameter.repeat_penalty.is_some() {
            parameters.push("repeat_penalty".to_string());
        }
        if parameter.top_k.is_some() {
            parameters.push("top_k".to_string());
        }
        if parameter.top_p.is_some() {
            parameters.push("top_p".to_string());
        }

        if capability.tool == Some(true) {
            parameters.push("tools".to_string());
        }
        if capability.json == Some(true) {
            parameters.push("structured_output".to_string());
        }
        if capability.reasoning.map(|value| value.is_enabled()) == Some(true) {
            parameters.push("reasoning".to_string());
        }

        parameters.sort();
        parameters.dedup();
        parameters
    }

    pub fn media_model_ids(&self) -> (Option<&str>, Option<&str>) {
        (
            self.model.config.media_gen.image_model.as_deref(),
            self.model.config.media_gen.video_model.as_deref(),
        )
    }

    fn media_model_prompt_parameters(&self) -> (Vec<String>, Vec<String>) {
        let (image_model_id, video_model_id) = self.media_model_ids();
        let mut image_parameters = Vec::new();
        let mut video_parameters = Vec::new();

        if image_model_id.is_some() {
            image_parameters.extend(
                ["prompt", "aspect_ratio", "reference_file_names"]
                    .into_iter()
                    .map(str::to_string),
            );
        }

        if let Some(video_model_id) = video_model_id {
            video_parameters.extend(
                [
                    "prompt",
                    "duration",
                    "resolution",
                    "aspect_ratio",
                    "size",
                    "generate_audio",
                    "reference_file_names",
                ]
                .into_iter()
                .map(str::to_string),
            );

            let _ = video_model_id;
            video_parameters.extend(
                [
                    "supported_resolutions",
                    "supported_aspect_ratios",
                    "supported_sizes",
                    "allowed_passthrough_parameters",
                ]
                .into_iter()
                .map(str::to_string),
            );
        }

        image_parameters.sort();
        image_parameters.dedup();
        video_parameters.sort();
        video_parameters.dedup();
        (image_parameters, video_parameters)
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
                    log::debug!("session halted: msg_id={}", self.message.id);
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
                        None => {
                            log::debug!("session stream exhausted: msg_id={}", self.message.id);
                            return Ok(StreamEndReason::Exhausted);
                        }
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
                let dimensions = imagesize::blob_size(&img.data).ok().map(|size| Dimensions {
                    width: size.width as i32,
                    height: size.height as i32,
                });
                self.message.inner.add_image(id, dimensions);
                self.publisher.publish(Token::Image(id));
            }
        }
    }

    async fn store_image(&self, img: &openrouter::GeneratedImage) -> Option<i32> {
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

    pub async fn store_blob_file(&mut self, img: &openrouter::GeneratedImage) -> Result<i32> {
        let size = img.data.len();

        use ::entity::file;
        let file_record = file::ActiveModel {
            chat_id: Set(Some(self.chat.id)),
            owner_id: Set(None),
            mime_type: Set(Some(img.mime_type.clone())),
            valid_until: Set(None),
            ..Default::default()
        };

        let file_id = file::Entity::insert(file_record)
            .exec(&self.ctx.db)
            .await?
            .last_insert_id;

        let byte_stream = tokio_stream::iter(vec![bytes::Bytes::from(img.data.clone())]);
        self.ctx.blob.insert(file_id, size, byte_stream).await?;
        self.set_file_mime_type(file_id, Some(img.mime_type.clone()));
        Ok(file_id)
    }

    pub async fn store_blob_video(
        &mut self,
        video: &mut openrouter::GeneratedVideo,
    ) -> Result<i32> {
        let mime_type = video
            .mime_type
            .clone()
            .unwrap_or_else(|| "video/mp4".to_string());

        use ::entity::file;
        let file_record = file::ActiveModel {
            chat_id: Set(Some(self.chat.id)),
            owner_id: Set(None),
            mime_type: Set(Some(mime_type.clone())),
            valid_until: Set(None),
            ..Default::default()
        };

        let file_id = file::Entity::insert(file_record)
            .exec(&self.ctx.db)
            .await?
            .last_insert_id;

        let insert_result = if let Some(size) = video
            .content_length
            .and_then(|value| usize::try_from(value).ok())
        {
            let chunk_stream = futures_util::stream::try_unfold(video, |video| async move {
                match video.next_chunk().await {
                    Ok(Some(chunk)) => Ok(Some((chunk, video))),
                    Ok(None) => Ok(None),
                    Err(error) => Err(error),
                }
            })
            .map_err(anyhow::Error::from);

            self.ctx
                .blob
                .insert_with_error(file_id, size, chunk_stream)
                .await
        } else {
            let temp_path = std::env::temp_dir().join(format!("llumen-video-{file_id}.tmp"));
            let mut temp_file = tokio::fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp_path)
                .await?;

            let mut size = 0usize;
            while let Some(chunk) = video.next_chunk().await? {
                size += chunk.len();
                temp_file.write_all(&chunk).await?;
            }
            temp_file.flush().await?;
            drop(temp_file);

            let temp_file = tokio::fs::OpenOptions::new()
                .read(true)
                .open(&temp_path)
                .await?;
            let chunk_stream = futures_util::stream::try_unfold(
                (temp_file, vec![0u8; 256 * 1024]),
                |(mut temp_file, mut buffer)| async move {
                    let read_bytes = temp_file.read(&mut buffer).await?;
                    if read_bytes == 0 {
                        return Ok::<
                            Option<(bytes::Bytes, (tokio::fs::File, Vec<u8>))>,
                            std::io::Error,
                        >(None);
                    }
                    Ok::<Option<(bytes::Bytes, (tokio::fs::File, Vec<u8>))>, std::io::Error>(Some(
                        (
                            bytes::Bytes::copy_from_slice(&buffer[..read_bytes]),
                            (temp_file, buffer),
                        ),
                    ))
                },
            )
            .map_err(anyhow::Error::from);

            let insert_result = self
                .ctx
                .blob
                .insert_with_error(file_id, size, chunk_stream)
                .await;
            if let Err(error) = tokio::fs::remove_file(&temp_path).await {
                log::warn!(
                    "failed to remove temp video file {}: {error}",
                    temp_path.display()
                );
            }
            insert_result
        };

        let insert_result = match insert_result {
            Ok(value) => value,
            Err(error) => {
                if let Err(delete_error) =
                    file::Entity::delete_by_id(file_id).exec(&self.ctx.db).await
                {
                    log::warn!("failed to delete file record {file_id}: {delete_error}");
                }
                return Err(error.into());
            }
        };
        if let Err(error) = insert_result {
            if let Err(delete_error) = file::Entity::delete_by_id(file_id).exec(&self.ctx.db).await
            {
                log::warn!("failed to delete file record {file_id}: {delete_error}");
            }
            return Err(error);
        }

        self.set_file_mime_type(file_id, Some(mime_type));

        Ok(file_id)
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
        let msg_id = self.message.id;
        let cost = self.cost;
        let token_count = self.token_count;

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

        log::debug!(
            "session saved: msg_id={}, cost={}, tokens={}",
            msg_id,
            cost,
            token_count
        );

        Ok(())
    }

    /// Syncs the session's model and mode to the chat record.
    pub async fn sync_chat_model(&mut self) -> Result<()> {
        let model_changed = self.chat.model_id != Some(self.model.id);
        let mode_changed = self.chat.mode != self.mode;

        if !model_changed && !mode_changed {
            return Ok(());
        }

        let mut chat_active: chat::ActiveModel = self.chat.clone().into();
        if model_changed {
            chat_active.model_id = Set(Some(self.model.id));
        }
        if mode_changed {
            chat_active.mode = Set(self.mode);
        }
        self.chat = chat::Entity::update(chat_active).exec(&self.ctx.db).await?;

        Ok(())
    }

    /// Generates and persists a chat title if one doesn't exist.
    /// Should be called after the completion finishes but before save().
    pub async fn try_generate_title(&mut self) -> Result<()> {
        if self.chat.title.is_some() {
            log::debug!("try_generate_title: chat already has title, skipping");
            return Ok(());
        }

        log::trace!("try_generate_title: attempting to generate title");

        let title = if let Some(generated_title) = self.generate_title().await {
            generated_title
        } else {
            // Fallback for image-only models: use trimmed user query (max 40 chars)
            log::trace!("try_generate_title: generation failed, using fallback");
            let user_msg = self.latest_user_message().unwrap_or("New Chat");
            let trimmed: String = user_msg.trim().chars().take(40).collect();
            if trimmed.is_empty() {
                "New Chat".to_string()
            } else {
                trimmed
            }
        };

        log::trace!("try_generate_title: using title: '{}'", title);
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
                files: Vec::new(),
            },
            openrouter::Message::User(
                "Please generate a concise title, starting with a emoji".to_string(),
            ),
        ];

        let mut model = self.openrouter_model();
        if let Some(ref task_model_id) = self.model.config.task_model_id {
            model.id.clone_from(task_model_id);
        }
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
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::blob::BlobDB;
    use migration::MigratorTrait;
    use futures_util::FutureExt;
    use tokio_stream::StreamExt;

    const MODEL_CONFIG: &str = "display_name = 'Test model'\nmodel_id = 'test/model'";

    async fn receive_tokens(
        mut subscriber: impl tokio_stream::Stream<Item = Token> + Unpin,
        count: usize,
    ) -> Result<Vec<Token>> {
        let tokens = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            (&mut subscriber).take(count).collect::<Vec<_>>(),
        )
        .await?;
        assert!(subscriber.next().now_or_never().is_none());
        Ok(tokens)
    }

    async fn context() -> Result<Arc<Context>> {
        let mut options = ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let database = Database::connect(options).await?;
        migration::Migrator::up(&database, None).await?;
        user::Entity::delete_many().exec(&database).await?;
        user::ActiveModel {
            id: Set(1),
            name: Set("test-user".to_string()),
            password: Set(String::new()),
            preference: Set(UserPreference {
                theme: None,
                locale: None,
                submit_on_enter: None,
            }),
        }
        .insert(&database)
        .await?;
        for id in [7, 8] {
            entity_model::ActiveModel {
                id: Set(id),
                config: Set(MODEL_CONFIG.to_string()),
            }
            .insert(&database)
            .await?;
        }
        chat::ActiveModel {
            id: Set(42),
            owner_id: Set(1),
            model_id: Set(Some(7)),
            mode: Set(ModeKind::Normal),
            title: Set(Some("Original title".to_string())),
        }
        .insert(&database)
        .await?;

        let blobs =
            redb::Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?;
        Ok(Arc::new(Context::new(
            database,
            Arc::new(openrouter::Openrouter::new("", "http://127.0.0.1:0", false)),
            Arc::new(BlobDB::new(Arc::new(blobs))),
        )?))
    }

    fn record(id: i32, inner: MessageInner) -> message::Model {
        message::Model {
            id,
            chat_id: 42,
            price: 0.0,
            token_count: 0,
            inner,
        }
    }

    fn user_message(text: &str) -> MessageInner {
        MessageInner::User {
            text: text.to_string(),
            files: Vec::new(),
        }
    }

    fn file_metadata(id: i32) -> FileMetadata {
        FileMetadata {
            id,
            name: format!("file-{id}"),
            kind: FileKind::User,
            dimensions: None,
        }
    }

    #[test]
    fn collects_unique_file_ids_from_user_messages_and_tool_results() {
        let history = [
            record(
                1,
                MessageInner::User {
                    text: "Attached files".to_string(),
                    files: vec![file_metadata(9), file_metadata(3), file_metadata(9)],
                },
            ),
            record(
                2,
                MessageInner::Assistant(vec![
                    AssistantChunk::Text("Response".to_string()),
                    AssistantChunk::ToolResult {
                        id: "tool-1".to_string(),
                        response: String::new(),
                        files: vec![file_metadata(3), file_metadata(7)],
                    },
                ]),
            ),
            record(3, user_message("No files")),
        ];
        assert_eq!(
            CompletionSession::collect_history_file_ids(&history),
            [3, 7, 9]
        );
        assert!(CompletionSession::collect_history_file_ids(&[]).is_empty());
    }

    #[tokio::test]
    async fn loads_ordered_history_and_creates_a_separate_assistant_placeholder() -> Result<()> {
        let context = context().await?;
        for message in [
            record(20, user_message("Latest")),
            record(10, user_message("First")),
        ] {
            message::ActiveModel::from(message)
                .insert(&context.db)
                .await?;
        }
        chat::ActiveModel {
            id: Set(43),
            owner_id: Set(1),
            model_id: Set(Some(7)),
            mode: Set(ModeKind::Normal),
            title: Set(None),
        }
        .insert(&context.db)
        .await?;
        message::ActiveModel {
            chat_id: Set(43),
            ..message::ActiveModel::from(record(30, user_message("Another chat")))
        }
        .insert(&context.db)
        .await?;
        let session = CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        assert_eq!(
            session
                .history
                .iter()
                .map(|message| message.id)
                .collect::<Vec<_>>(),
            [10, 20]
        );
        assert_eq!(session.latest_user_message(), Some("Latest"));
        assert!(session.message.id > 20);
        assert_eq!(session.message.inner, MessageInner::default());
        assert!(!context.channel.publishable(42));
        drop(session);
        assert!(context.channel.publishable(42));
        Ok(())
    }

    #[tokio::test]
    async fn rejects_a_second_session_without_inserting_an_orphan_message() -> Result<()> {
        let context = context().await?;
        let session = CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let count = message::Entity::find().count(&context.db).await?;
        let second = CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await;
        let error = second.err().context("second session was accepted")?;
        assert!(error.to_string().contains("already streaming"));
        assert_eq!(message::Entity::find().count(&context.db).await?, count);
        drop(session);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_missing_records_before_creating_an_assistant_message() -> Result<()> {
        let context = context().await?;
        for (user_id, chat_id, model_id, expected) in [
            (99, 42, 7, "user not found"),
            (1, 99, 7, "chat not found"),
            (1, 42, 99, "model not found"),
        ] {
            let result = CompletionSession::new(
                context.clone(),
                user_id,
                chat_id,
                model_id,
                ModeKind::Normal,
            )
            .await;
            assert_eq!(
                result
                    .err()
                    .context("invalid session was accepted")?
                    .to_string(),
                expected
            );
            assert_eq!(message::Entity::find().count(&context.db).await?, 0);
            assert!(context.channel.publishable(42));
        }
        Ok(())
    }

    #[tokio::test]
    async fn releases_the_streaming_channel_when_placeholder_insertion_fails() -> Result<()> {
        let context = context().await?;
        context
            .db
            .execute_unprepared(
                "CREATE TRIGGER reject_message BEFORE INSERT ON message
             BEGIN SELECT RAISE(ABORT, 'message insertion failed'); END;",
            )
            .await?;
        let result = CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await;
        assert!(
            result
                .err()
                .context("message insertion succeeded")?
                .to_string()
                .contains("message insertion failed")
        );
        assert_eq!(message::Entity::find().count(&context.db).await?, 0);
        assert!(context.channel.publishable(42));
        Ok(())
    }

    #[tokio::test]
    async fn loads_mime_types_for_history_files_and_updates_existing_entries() -> Result<()> {
        let context = context().await?;
        for (id, mime_type) in [(3, Some("image/png")), (7, None)] {
            file::ActiveModel {
                id: Set(id),
                chat_id: Set(Some(42)),
                owner_id: Set(Some(1)),
                mime_type: Set(mime_type.map(str::to_string)),
                valid_until: Set(None),
            }
            .insert(&context.db)
            .await?;
        }
        message::ActiveModel::from(record(
            1,
            MessageInner::User {
                text: "Files".to_string(),
                files: vec![file_metadata(3), file_metadata(7), file_metadata(99)],
            },
        ))
        .insert(&context.db)
        .await?;
        let mut session = CompletionSession::new(context, 1, 42, 7, ModeKind::Normal).await?;
        assert_eq!(session.file_mime_type(3), Some("image/png"));
        assert_eq!(session.file_mime_type(7), None);
        assert_eq!(session.file_mime_type(99), None);
        session.set_file_mime_type(3, Some("image/webp".to_string()));
        session.set_file_mime_type(9, Some("video/mp4".to_string()));
        assert_eq!(session.file_mime_type(3), Some("image/webp"));
        assert_eq!(session.file_mime_type(9), Some("video/mp4"));
        assert_eq!(
            session
                .file_mime_types
                .iter()
                .filter(|(id, _)| *id == 3)
                .count(),
            1
        );
        Ok(())
    }

    #[tokio::test]
    async fn latest_user_message_skips_assistant_messages_and_respects_locale() -> Result<()> {
        let mut session =
            CompletionSession::new(context().await?, 1, 42, 7, ModeKind::Normal).await?;
        assert_eq!(session.latest_user_message(), None);
        assert_eq!(session.locale(), "en-US");
        session.history = vec![
            record(1, user_message("First")),
            record(2, user_message("Latest")),
            record(
                3,
                MessageInner::Assistant(vec![AssistantChunk::Text("Assistant reply".to_string())]),
            ),
        ];
        session.user.preference.locale = Some("zh-TW".to_string());
        assert_eq!(session.latest_user_message(), Some("Latest"));
        assert_eq!(session.locale(), "zh-TW");
        Ok(())
    }

    #[tokio::test]
    async fn inserts_context_before_the_last_user_query_without_reordering_history() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        session.history = vec![
            record(1, user_message("First query")),
            record(
                2,
                MessageInner::Assistant(vec![AssistantChunk::Text("First reply".to_string())]),
            ),
            record(3, user_message("Latest query")),
        ];
        let messages =
            session.assemble_messages(&context, openrouter::CompletionOption::default())?;
        assert_eq!(messages.len(), 5);
        assert!(matches!(
            messages.first(),
            Some(openrouter::Message::System(_))
        ));
        assert!(
            matches!(messages.get(1), Some(openrouter::Message::User(text)) if text == "First query")
        );
        assert!(
            matches!(messages.get(2), Some(openrouter::Message::Assistant { content, .. }) if content == "First reply")
        );
        assert!(
            matches!(messages.get(3), Some(openrouter::Message::User(text)) if !text.trim().is_empty())
        );
        assert!(
            matches!(messages.last(), Some(openrouter::Message::User(text)) if text == "Latest query")
        );
        Ok(())
    }

    #[tokio::test]
    async fn saves_chunks_and_accumulated_usage_before_emitting_completion() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let message_id = session.message.id;
        let subscriber = context.clone().subscribe(42, None);
        session.add_chunk(AssistantChunk::Text("Hello".to_string()));
        session.extend_chunks(vec![AssistantChunk::Reasoning("Thinking".to_string())]);
        session.update_usage(0.25, 10);
        session.update_usage(0.5, 20);
        let expected = session.message.inner.clone();
        session.save().await?;

        let saved = message::Entity::find_by_id(message_id)
            .one(&context.db)
            .await?
            .context("saved message missing")?;
        assert_eq!(saved.inner, expected);
        assert_eq!(saved.price, 0.75);
        assert_eq!(saved.token_count, 30);
        let tokens = receive_tokens(subscriber, 1).await?;
        assert!(
            matches!(tokens.as_slice(), [Token::Complete { message_id: id, cost, token: 30 }] if *id == message_id && *cost == 0.75)
        );
        assert!(context.channel.publishable(42));
        Ok(())
    }

    #[tokio::test]
    async fn does_not_emit_completion_when_persistence_fails() -> Result<()> {
        let context = context().await?;
        let session = CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let subscriber = context.clone().subscribe(42, None);
        message::Entity::delete_by_id(session.message.id)
            .exec(&context.db)
            .await?;
        assert!(session.save().await.is_err());
        assert!(receive_tokens(subscriber, 0).await?.is_empty());
        assert!(context.channel.publishable(42));
        Ok(())
    }

    #[tokio::test]
    async fn synchronizes_model_and_mode_without_overwriting_the_title() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 8, ModeKind::Search).await?;
        session.sync_chat_model().await?;
        let saved = chat::Entity::find_by_id(42)
            .one(&context.db)
            .await?
            .context("chat missing")?;
        assert_eq!(saved.model_id, Some(8));
        assert_eq!(saved.mode, ModeKind::Search);
        assert_eq!(saved.title.as_deref(), Some("Original title"));
        assert_eq!(session.chat, saved);
        Ok(())
    }

    #[tokio::test]
    async fn preserves_local_chat_state_when_model_synchronization_fails() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 8, ModeKind::Search).await?;
        let original = session.chat.clone();
        chat::Entity::delete_by_id(42).exec(&context.db).await?;
        assert!(session.sync_chat_model().await.is_err());
        assert_eq!(session.chat, original);
        Ok(())
    }

    #[tokio::test]
    async fn skips_model_synchronization_and_title_generation_when_unchanged() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        chat::Entity::delete_by_id(42).exec(&context.db).await?;
        session.sync_chat_model().await?;
        session.try_generate_title().await?;
        assert_eq!(session.chat.title.as_deref(), Some("Original title"));
        Ok(())
    }

    #[tokio::test]
    async fn adds_errors_to_both_the_live_stream_and_stored_message() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let subscriber = context.clone().subscribe(42, None);
        session.add_error("Provider unavailable".to_string());
        assert_eq!(
            session.message.inner,
            MessageInner::Assistant(vec![AssistantChunk::Error(
                "Provider unavailable".to_string()
            )])
        );
        drop(session);
        assert!(
            matches!(receive_tokens(subscriber, 1).await?.as_slice(), [Token::Error(error)] if error == "Provider unavailable")
        );
        Ok(())
    }

    #[tokio::test]
    async fn publishes_stream_tokens_and_reports_upstream_errors() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let subscriber = context.clone().subscribe(42, None);
        let reason = session
            .put_stream(tokio_stream::iter([
                Ok(Token::Assistant("Partial answer".to_string())),
                Err(openrouter::Error::MalformedResponse("missing content")),
                Ok(Token::Assistant("Must not appear".to_string())),
            ]))
            .await?;
        assert!(matches!(reason, StreamEndReason::Exhausted));
        drop(session);
        let tokens = receive_tokens(subscriber, 2).await?;
        assert!(
            matches!(tokens.as_slice(), [Token::Assistant(text), Token::Error(error)] if text == "Partial answer" && error.contains("missing content"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn halts_while_waiting_for_an_upstream_token() -> Result<()> {
        let context = context().await?;
        let mut session =
            CompletionSession::new(context.clone(), 1, 42, 7, ModeKind::Normal).await?;
        let (reason, ()) = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            tokio::join!(
                async {
                    let reason = session.put_stream(tokio_stream::pending()).await;
                    drop(session);
                    reason
                },
                context.halt_session(42),
            )
        })
        .await?;
        assert!(matches!(reason?, StreamEndReason::Halt));
        assert!(context.channel.publishable(42));
        Ok(())
    }
}
