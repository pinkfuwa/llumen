use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;
use serde::Deserialize;

use super::model_strategy;
use super::{ExecutionStrategy, RunState};
use crate::chat::prompt;
use crate::chat::tools::{get_crawl_tool_def, get_web_search_tool_def};
use crate::chat::Context;
use crate::openrouter::{self, Capability, ToolCall};

/// Search mode: web search + crawl tools, with OpenRouter plugin support.
pub struct SearchStrategy;

impl ExecutionStrategy for SearchStrategy {
    fn prompt_kind(&self) -> prompt::PromptKind {
        prompt::PromptKind::Search
    }

    fn completion_option(
        &self,
        ctx: &Context,
        capability: &Capability,
    ) -> openrouter::CompletionOption {
        let is_openrouter = !ctx.openrouter.is_compatibility_mode();

        let mut tools = vec![get_web_search_tool_def(), get_crawl_tool_def()];

        // On native OpenRouter, the platform handles web search via plugins,
        // so we don't need the tool-call versions.
        if is_openrouter {
            tools.retain(|t| t.name != "web_search_tool" && t.name != "crawl_tool");
        }

        // Let model strategy filter tools (image-only models get no tools)
        let strategy = model_strategy::get_model_strategy(capability);
        tools = strategy.filter_tools(tools);

        openrouter::CompletionOption::builder()
            .web_search(true)
            .tools(&tools)
            .build()
    }

    fn handle_tool_calls<'a>(
        &'a self,
        state: &'a mut RunState,
        toolcalls: Vec<ToolCall>,
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let assistant_chunks = state.session.message.inner.as_assistant().unwrap();
            let assistant_text = assistant_chunks
                .iter()
                .filter_map(|chunk| {
                    if let protocol::AssistantChunk::Text(text) = chunk {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            let annotations = assistant_chunks.iter().rev().find_map(|chunk| {
                if let protocol::AssistantChunk::Annotation(value) = chunk {
                    Some(value.clone())
                } else {
                    None
                }
            });

            if !assistant_text.is_empty() {
                state.messages.push(openrouter::Message::Assistant {
                    content: assistant_text,
                    annotations,
                    reasoning_details: None,
                    images: Vec::new(),
                });
            }

            for toolcall in toolcalls {
                let result =
                    execute_search_tool(&state.ctx, &toolcall.name, &toolcall.args).await;

                state
                    .session
                    .message
                    .inner
                    .as_assistant()
                    .unwrap()
                    .push(protocol::AssistantChunk::ToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    });

                state
                    .session
                    .add_token(crate::chat::token::Token::ToolCall {
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    });

                state.messages.push(openrouter::Message::ToolCall(
                    openrouter::MessageToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arguments: toolcall.args.clone(),
                    },
                ));

                state
                    .session
                    .message
                    .inner
                    .as_assistant()
                    .unwrap()
                    .push(protocol::AssistantChunk::ToolResult {
                        id: toolcall.id.clone(),
                        response: result.clone(),
                    });

                state
                    .session
                    .add_token(crate::chat::token::Token::ToolResult(result.clone()));

                state.messages.push(openrouter::Message::ToolResult(
                    openrouter::MessageToolResult {
                        id: toolcall.id.clone(),
                        content: result,
                    },
                ));
            }

            Ok(false) // Not finalized — runner will loop for more LLM output
        })
    }
}

async fn execute_search_tool(ctx: &Arc<Context>, tool_name: &str, args: &str) -> String {
    match tool_name {
        "web_search_tool" => {
            #[derive(Deserialize)]
            struct WebSearchArgs {
                query: String,
            }
            let args: Option<WebSearchArgs> = serde_json::from_str(args).ok();
            if args.is_none() {
                return "Invalid arguments for web_search_tool".to_string();
            }
            let args = args.unwrap();
            match ctx.web_search_tool.search(&args.query).await {
                Ok(results) => {
                    let mut output = String::new();
                    for (i, result) in results.iter().enumerate().take(10) {
                        output.push_str(&format!(
                            "{}. [{}]({})\n   {}\n\n",
                            i + 1,
                            result.title,
                            result.url,
                            result.description
                        ));
                    }

                    if output.is_empty() {
                        output = "No search results found.".to_string();
                    }

                    output
                }
                Err(e) => {
                    log::warn!("Web search error: {}", e);
                    format!("Error: {}", e)
                }
            }
        }
        "crawl_tool" => {
            #[derive(Deserialize)]
            struct CrawlArgs {
                url: String,
            }
            let args: Option<CrawlArgs> = serde_json::from_str(args).ok();
            if args.is_none() {
                return "Invalid arguments for crawl_tool".to_string();
            }
            let args = args.unwrap();
            match ctx.crawl_tool.crawl(&args.url).await {
                Ok(content) => content,
                Err(e) => {
                    log::warn!("Crawl error for URL '{}': {}", args.url, e);
                    format!("Error: {}", e)
                }
            }
        }
        _ => format!("Unknown tool: {}", tool_name),
    }
}
