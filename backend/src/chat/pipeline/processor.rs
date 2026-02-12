use anyhow::Result;
use bytes::Bytes;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use tokio_stream::StreamExt;

use super::runner::RunState;
use crate::chat::converter::openrouter_stream_to_assitant_chunk;
use crate::chat::token::Token;
use crate::openrouter;
use entity::file;

/// Handles post-processing of LLM stream results.
///
/// After streaming tokens to the user, we need to:
/// - Update usage stats (cost, tokens)
/// - Store assistant chunks (text, reasoning)
/// - Store generated images (DB + blob storage)
/// - Store annotations (citations)
///
/// This is common infrastructure used by all strategies.
pub struct StreamProcessor;

impl StreamProcessor {
    /// Process the full result after streaming completes.
    pub async fn process_result(
        state: &mut RunState,
        result: &openrouter::StreamResult,
    ) -> Result<()> {
        Self::update_usage(state, result);
        Self::store_chunks(state, result);
        Self::store_reasoning(state, result);
        Self::store_images(state, result).await?;
        Self::store_annotations(state, result);
        Ok(())
    }

    /// Update usage statistics (cost and token count).
    fn update_usage(state: &mut RunState, result: &openrouter::StreamResult) {
        state
            .session
            .update_usage(result.usage.cost as f32, result.usage.token as i32);
    }

    /// Store assistant text chunks on the message entity.
    fn store_chunks(state: &mut RunState, result: &openrouter::StreamResult) {
        state
            .session
            .message
            .inner
            .as_assistant()
            .unwrap()
            .extend(openrouter_stream_to_assitant_chunk(&result.responses));
    }

    /// Store reasoning details if present (for o1-style models).
    fn store_reasoning(state: &mut RunState, result: &openrouter::StreamResult) {
        if let Some(ref reasoning_details) = result.reasoning_details {
            state
                .session
                .message
                .inner
                .add_reasoning_detail(reasoning_details.clone());
        }
    }

    /// Store generated images: save to blob storage + DB.
    async fn store_images(state: &mut RunState, result: &openrouter::StreamResult) -> Result<()> {
        if result.image.is_empty() {
            return Ok(());
        }

        for image in &result.image {
            let chat_id = state.session.get_chat_id();
            let mime_type = image.mime_type.clone();
            let owner_id = state.session.user.id;
            let data = image.data.clone();
            let blob = state.ctx.blob.clone();
            let db = &state.ctx.db;

            // Insert file record into DB
            let file_record = file::ActiveModel {
                chat_id: ActiveValue::Set(Some(chat_id)),
                owner_id: ActiveValue::Set(Some(owner_id)),
                mime_type: ActiveValue::Set(Some(mime_type)),
                ..Default::default()
            }
            .insert(db)
            .await?;

            let file_id = file_record.id;
            let size = data.len();

            // Store blob data
            if let Err(e) = blob
                .insert(file_id, size, tokio_stream::once(Bytes::from(data)))
                .await
            {
                log::error!("Failed to store image in blob: {}", e);
                continue;
            }

            // Add to message and stream to client
            state.session.message.inner.add_image(file_id);
            state.session.add_token(Token::Image(file_id));
        }

        Ok(())
    }

    /// Store URL citations from annotations.
    fn store_annotations(state: &mut RunState, result: &openrouter::StreamResult) {
        if let Some(ref annotations) = result.annotations {
            let citations = openrouter::extract_url_citations(annotations);
            state.session.message.inner.add_annotation(annotations.clone());

            if !citations.is_empty() {
                state
                    .session
                    .message
                    .inner
                    .add_url_citation(citations.clone());
                state.session.add_token(Token::UrlCitation(citations));
            }
        }
    }
}
