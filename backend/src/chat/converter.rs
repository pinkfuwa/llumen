use std::sync::Arc;

use protocol::{AssistantChunk, FileMetadata, MessageInner};

use crate::{chat::Context, openrouter, utils::blob::BlobDB};
use anyhow::Result;

async fn load_files(
    db: Arc<BlobDB>,
    handles: &[FileMetadata],
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

pub async fn db_message_to_openrouter(
    ctx: &Context,
    message: &MessageInner,
) -> Result<impl Iterator<Item = openrouter::Message>> {
    let mut result = Vec::new();
    match message {
        MessageInner::User { text, files } => todo!(),
        MessageInner::Assistant(assistant_chunks) => {
            for chunk in assistant_chunks {
                match chunk {
                    AssistantChunk::Annotation(_) => todo!(),
                    AssistantChunk::Text(x) => {
                        result.push(openrouter::Message::Assistant(x.clone()))
                    }
                    AssistantChunk::Reasoning(_) => todo!(),
                    AssistantChunk::ToolCall { id, arg, name } => todo!(),
                    AssistantChunk::ToolResult { id, response } => todo!(),
                    AssistantChunk::Error(_) => todo!(),
                    AssistantChunk::DeepAgent(deep) => todo!(),
                }
            }
        }
    };
    Ok(result.into_iter())
}

pub fn openrouter_stream_to_assitant_chunk(
    msgs: &[openrouter::StreamCompletionResp],
) -> impl Iterator<Item = AssistantChunk> {
    let mut result = Vec::new();
    for msg in msgs {
        match msg {
            openrouter::StreamCompletionResp::ReasoningToken(x) => {
                if let Some(AssistantChunk::Reasoning(reasoning)) = result.last_mut() {
                    reasoning.push_str(x.as_str());
                } else {
                    result.push(AssistantChunk::Reasoning(x.clone()));
                }
            }
            openrouter::StreamCompletionResp::ResponseToken(x) => {
                if let Some(AssistantChunk::Text(response)) = result.last_mut() {
                    response.push_str(x.as_str());
                } else {
                    result.push(AssistantChunk::Text(x.clone()));
                }
            }
            openrouter::StreamCompletionResp::ToolCall { name, args, id } => todo!(),
            openrouter::StreamCompletionResp::ToolToken(_) => todo!(),
            openrouter::StreamCompletionResp::Usage { price, token } => {}
        }
    }
    result.into_iter()
}
