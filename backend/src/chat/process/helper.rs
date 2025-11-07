use std::sync::Arc;

use entity::{FileHandle, chunk};
use sea_orm::IntoActiveModel;

use crate::{
    chat::token::ToolCallInfo,
    openrouter::{self, MessageToolCall},
    utils::blob::BlobDB,
};

/// Convert **assistant** chunks to openrouter messages
pub(super) fn chunks_to_message(
    chunks: impl Iterator<Item = chunk::Model> + 'static,
) -> Vec<crate::openrouter::Message> {
    active_chunks_to_message(chunks.map(|c| c.into_active_model()))
}

/// Convert **assistant** chunks to openrouter messages
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
            entity::ChunkKind::Annotation => {
                let annotations: serde_json::Value = serde_json::from_str(&content).unwrap();
                if let Some(openrouter::Message::Assistant(last)) = results.pop() {
                    results.push(openrouter::Message::AssistantAnnotationed {
                        text: last,
                        annotations,
                    });
                } else {
                    log::warn!("Annotation chunk without preceding text chunk");
                }
            }
            _ => continue,
        };
    }
    results
}

pub(super) async fn load_files(
    db: Arc<BlobDB>,
    handles: &[FileHandle],
) -> Result<Vec<openrouter::File>, anyhow::Error> {
    let mut tasks = Vec::with_capacity(handles.len());

    for handle in handles {
        let id = handle.id;
        let name = handle.name.clone();
        let db = db.clone();
        let handle = tokio::spawn(async move {
            db.get_vectored(id)
                .await
                .map(|data| openrouter::File { name, data })
        });
        tasks.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for task in tasks {
        match task.await? {
            Some(it) => results.push(it),
            None => log::error!("File not found"),
        };
    }

    Ok(results)
}
