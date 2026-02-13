//! Normal chat mode – no tools, single completion round.

use anyhow::Result;
use tokio_stream::StreamExt;

use crate::chat::context::StreamEndReason;
use crate::chat::converter::openrouter_to_buffer_token;
use crate::chat::session::CompletionSession;
use crate::chat::Context;
use crate::openrouter;

pub async fn execute(ctx: &Context, session: &mut CompletionSession) -> Result<()> {
    let messages = session.assemble_messages(ctx, openrouter::CompletionOption::default())?;

    let model = session.openrouter_model();

    let mut stream: openrouter::StreamCompletion = ctx
        .openrouter
        .stream(model, messages, openrouter::CompletionOption::default())
        .await?;

    let halt = session
        .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token)))
        .await?;

    let result = stream.get_result();
    session.update_usage(result.usage.cost as f32, result.usage.token as i32);

    // Convert stream responses to assistant chunks and persist them
    let chunks = crate::chat::converter::openrouter_stream_to_assitant_chunk(&result.responses);
    session.extend_chunks(chunks);

    // Persist annotations / reasoning details / images
    session.apply_stream_result(&result).await;

    if matches!(halt, StreamEndReason::Halt) {
        return Ok(());
    }

    Ok(())
}
