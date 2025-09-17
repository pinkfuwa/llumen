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
            entity::ChunkKind::Text => todo!(),
            entity::ChunkKind::Reasoning => todo!(),
            entity::ChunkKind::ToolCall => todo!(),
            entity::ChunkKind::Error => todo!(),
            entity::ChunkKind::Report => todo!(),
            entity::ChunkKind::Plan => todo!(),
            entity::ChunkKind::Step => todo!(),
        }
    }
    results
}

pub(super) fn to_token_stream<E>(
    stream: impl Stream<Item = Result<StreamCompletionResp, E>>,
) -> impl Stream<Item = Result<Token, E>> {
    stream.map(|res| {
        res.map(|resp| match resp {
            StreamCompletionResp::ReasoningToken(reasoning) => Token::Reasoning(reasoning),
            StreamCompletionResp::ResponseToken(content) => Token::Message(content),
            StreamCompletionResp::ToolCall { name, args, id } => Token::Tool { name, args },
            _ => Token::Empty,
        })
    })
}
