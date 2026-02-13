//! Search chat mode – web_search + crawl tools with tool-call loop.

use anyhow::Result;
use protocol::AssistantChunk;
use tokio_stream::StreamExt;

use crate::chat::context::StreamEndReason;
use crate::chat::converter::{openrouter_stream_to_assitant_chunk, openrouter_to_buffer_token};
use crate::chat::session::CompletionSession;
use crate::chat::token::Token;
use crate::chat::Context;
use crate::openrouter;

pub async fn execute(ctx: &Context, session: &mut CompletionSession) -> Result<()> {
    let tools = ctx.tools.for_search_mode();
    let option = openrouter::CompletionOption::tools(&tools);
    let mut messages = session.assemble_messages(ctx, option.clone())?;

    loop {
        let model = session.openrouter_model();
        let mut stream: openrouter::StreamCompletion = ctx
            .openrouter
            .stream(model, messages.clone(), option.clone())
            .await?;

        let halt = session
            .put_stream((&mut stream).map(|resp| resp.map(openrouter_to_buffer_token)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            return Ok(());
        }

        let mut result = stream.get_result();
        session.update_usage(result.usage.cost as f32, result.usage.token as i32);

        session.apply_stream_result(&result);

        let tool_calls = std::mem::take(&mut result.toolcalls);
        let assistant_text = result.get_text();

        // Persist intermediate text
        let chunks = openrouter_stream_to_assitant_chunk(&result.responses);
        session.extend_chunks(chunks);

        if tool_calls.is_empty() {
            break;
        }

        // Re-add assistant turn so the model sees its own tool calls
        messages.push(openrouter::Message::Assistant {
            content: assistant_text,
            annotations: None,
            reasoning_details: None,
            images: Vec::new(),
        });

        for tc in tool_calls {
            messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arguments: tc.args.clone(),
            }));

            session.add_chunk(AssistantChunk::ToolCall {
                id: tc.id.clone(),
                name: tc.name.clone(),
                arg: tc.args.clone(),
            });

            let tool_result = execute_tool(ctx, &tc.name, &tc.args).await;

            messages.push(openrouter::Message::ToolResult(
                openrouter::MessageToolResult {
                    id: tc.id.clone(),
                    content: tool_result.clone(),
                },
            ));

            session.add_token(Token::ToolResult(tool_result.clone()));
            session.add_chunk(AssistantChunk::ToolResult {
                id: tc.id,
                response: tool_result,
            });
        }
    }

    Ok(())
}

async fn execute_tool(ctx: &Context, name: &str, args: &str) -> String {
    match name {
        "web_search_tool" => {
            #[derive(serde::Deserialize)]
            struct Args {
                query: String,
            }
            let parsed: Option<Args> = serde_json::from_str(args).ok();
            match parsed {
                None => "Invalid arguments for web_search_tool".to_string(),
                Some(a) => match ctx.tools.web_search.search(&a.query).await {
                    Ok(results) => {
                        let mut out = String::new();
                        for (i, r) in results.iter().enumerate().take(10) {
                            out.push_str(&format!(
                                "{}. [{}]({})\n   {}\n\n",
                                i + 1,
                                r.title,
                                r.url,
                                r.description
                            ));
                        }
                        if out.is_empty() {
                            "No search results found.".to_string()
                        } else {
                            out
                        }
                    }
                    Err(e) => {
                        log::warn!("Web search error: {}", e);
                        format!("Error: {}", e)
                    }
                },
            }
        }
        "crawl_tool" => {
            #[derive(serde::Deserialize)]
            struct Args {
                url: String,
            }
            let parsed: Option<Args> = serde_json::from_str(args).ok();
            match parsed {
                None => "Invalid arguments".to_string(),
                Some(a) => match ctx.tools.crawl.crawl(&a.url).await {
                    Ok(content) => content,
                    Err(e) => {
                        log::warn!("Crawl error for URL '{}': {}", a.url, e);
                        format!("Error: {}", e)
                    }
                },
            }
        }
        _ => format!("Unknown tool: {}", name),
    }
}
