use entity::chunk;
use sea_orm::IntoActiveModel;

use crate::{
    chat::token::ToolCallInfo,
    openrouter::{self, MessageToolCall},
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
        if chunk.content.is_not_set() {
            continue;
        }
        let content = chunk.content.unwrap();
        match chunk.kind.unwrap() {
            entity::ChunkKind::Text => results.push(openrouter::Message::Assistant(content)),
            entity::ChunkKind::ToolCall => {
                let info: ToolCallInfo = serde_json::from_str(&content).unwrap();
                results.push(openrouter::Message::ToolCall(MessageToolCall {
                    id: info.id.clone(),
                    name: info.name,
                    arguments: info.input,
                }));
                if let Some(output) = info.output {
                    results.push(openrouter::Message::ToolResult(
                        openrouter::MessageToolResult {
                            id: info.id,
                            content: output,
                        },
                    ));
                }
            }
            _ => continue,
        };
    }
    results
}
