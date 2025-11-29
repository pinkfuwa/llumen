use crate::chat::tools::{get_crawl_tool_def, get_web_search_tool_def};
use crate::{
    chat::{CompletionContext, Context, agent::chat::ChatInner, prompt::PromptKind},
    openrouter,
    utils::model::ModelChecker,
};
use anyhow::{Context as _, Result};
use futures_util::future::BoxFuture;
use protocol::ModelConfig;
use serde::Deserialize;

pub struct Inner;

impl ChatInner for Inner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String> {
        ctx.prompt
            .render(PromptKind::Search, completion_ctx)
            .context("Failed to render system prompt")
    }

    fn get_model(_: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model> {
        let mut model: openrouter::Model =
            <ModelConfig as ModelChecker>::from_toml(&completion_ctx.model.config)
                .context("Failed to get model config")?
                .into();
        model.online = true;

        Ok(model)
    }

    fn get_tools(
        _ctx: &Context,
        _completion_ctx: &CompletionContext,
    ) -> Result<Vec<openrouter::Tool>> {
        Ok(vec![get_web_search_tool_def(), get_crawl_tool_def()])
    }

    fn handoff_tool<'a>(
        pipeline: &'a mut crate::chat::agent::chat::ChatPipeline<Self>,
        toolcalls: Vec<openrouter::ToolCall>,
    ) -> BoxFuture<'a, Result<bool, anyhow::Error>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let assistant_text = pipeline
                .completion_ctx
                .message
                .inner
                .as_assistant()
                .unwrap()
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

            if !assistant_text.is_empty() {
                pipeline.messages.push(openrouter::Message::Assistant {
                    content: assistant_text,
                    annotations: None,
                    reasoning_details: None,
                });
            }

            for toolcall in toolcalls {
                let result =
                    execute_search_tool(&pipeline.ctx, &toolcall.name, &toolcall.args).await;

                pipeline
                    .completion_ctx
                    .message
                    .inner
                    .as_assistant()
                    .unwrap()
                    .push(protocol::AssistantChunk::ToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    });

                pipeline
                    .completion_ctx
                    .add_token_force(crate::chat::token::Token::ToolCall {
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    });

                pipeline.messages.push(openrouter::Message::ToolCall(
                    openrouter::MessageToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arguments: toolcall.args.clone(),
                    },
                ));

                pipeline
                    .completion_ctx
                    .message
                    .inner
                    .as_assistant()
                    .unwrap()
                    .push(protocol::AssistantChunk::ToolResult {
                        id: toolcall.id.clone(),
                        response: result.clone(),
                    });

                pipeline
                    .completion_ctx
                    .add_token_force(crate::chat::token::Token::ToolResult(result.clone()));

                pipeline.messages.push(openrouter::Message::ToolResult(
                    openrouter::MessageToolResult {
                        id: toolcall.id.clone(),
                        content: result,
                    },
                ));
            }

            Ok(false)
        })
    }
}

async fn execute_search_tool(ctx: &std::sync::Arc<Context>, tool_name: &str, args: &str) -> String {
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
