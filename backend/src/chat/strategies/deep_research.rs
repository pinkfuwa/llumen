//! Deep Research mode – coordinator triggers deep agent via tool call.

use anyhow::Result;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::chat::context::StreamEndReason;
use crate::chat::converter::openrouter_to_buffer_token;
use crate::chat::session::CompletionSession;
use crate::chat::Context;
use crate::openrouter;

pub async fn execute(ctx: Arc<Context>, session: &mut CompletionSession) -> Result<()> {
    // Assemble messages with coordinator prompt
    let messages = session.assemble_messages(&ctx, openrouter::CompletionOption::default())?;

    let model = session.openrouter_model();

    // Get the deep research tool definition
    let deep_tool = ctx.tools.get_deep_research_def();
    let option = openrouter::CompletionOption::tools(&[deep_tool]);

    // Stream the coordinator's response
    let mut stream: openrouter::StreamCompletion =
        ctx.openrouter.stream(model, messages, option).await?;

    let halt = session
        .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token)))
        .await?;

    if matches!(halt, StreamEndReason::Halt) {
        return Ok(());
    }

    let result = stream.get_result();
    session.update_usage(result.usage.cost as f32, result.usage.token as i32);

    // Check if coordinator called the deep research tool
    if let Some(tool_call) = result.toolcalls.first() {
        if tool_call.name == "handoff_to_planner" {
            // Hand off to the deep research agent
            super::super::deep_research::DeepAgent::handoff_tool_static(
                &ctx,
                session,
                result.toolcalls.clone(),
            )
            .await?;
        } else {
            log::warn!("coordinator called unexpected tool: {}", tool_call.name);
        }
    } else {
        log::warn!("coordinator did not call handoff_to_planner tool");
    }

    Ok(())
}
