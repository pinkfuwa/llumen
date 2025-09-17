use entity::chunk;
use futures_util::{Stream, StreamExt};
use openrouter::StreamCompletionResp;
use sea_orm::IntoActiveModel;

use crate::{
    chat::{context, token::Token},
    openrouter,
};

/// Convert **assitant** chunks to openrouter messages
pub(super) fn chunks_to_message(
    chunks: impl Iterator<Item = chunk::Model> + 'static,
) -> Vec<crate::openrouter::Message> {
    active_chunks_to_message(chunks.map(|c| c.into_active_model()))
}

/// Convert **assitant** chunks to openrouter messages
pub(super) fn active_chunks_to_message(
    chunks: impl Iterator<Item = chunk::ActiveModel> + 'static,
) -> Vec<crate::openrouter::Message> {
    let mut results = vec![];
    for chunk in chunks {
        match chunk.kind.unwrap() {
            entity::ChunkKind::Text => todo!("Convert to openrouter::Message::Assistant"),
            entity::ChunkKind::Reasoning => todo!("Decide how to handle reasoning chunks"),
            entity::ChunkKind::ToolCall => todo!("Convert to openrouter::Message::ToolCall"),
            entity::ChunkKind::Error => todo!("Decide how to handle error chunks"),
            entity::ChunkKind::Report => todo!("Decide how to handle report chunks"),
            entity::ChunkKind::Plan => todo!("Decide how to handle plan chunks"),
            entity::ChunkKind::Step => todo!("Decide how to handle step chunks"),
        }
    }
    results
}
