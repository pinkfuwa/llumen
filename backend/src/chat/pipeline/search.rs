use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;

use super::model_strategy;
use super::{ExecutionStrategy, RunState};
use crate::chat::Context;
use crate::chat::prompt;
use crate::openrouter::{self, Capability, ToolCall};

/// Search mode: web search + crawl tools, with OpenRouter plugin support.
pub struct SearchStrategy;

impl ExecutionStrategy for SearchStrategy {
    fn prompt_kind(&self) -> prompt::PromptKind {
        prompt::PromptKind::Search
    }

    fn completion_option(
        &self,
        _ctx: &Context,
        _capability: &Capability,
    ) -> openrouter::CompletionOption {
        // Synchronous path returns empty tools — async prepare() fills them
        openrouter::CompletionOption::default()
    }

    fn prepare<'a>(
        &'a self,
        ctx: &'a Context,
        session: &'a crate::chat::CompletionSession,
        capability: &'a Capability,
    ) -> BoxFuture<'a, Result<super::Execution>> {
        use crate::chat::converter::db_message_to_openrouter;
        use crate::chat::pipeline::message_builder::MessageBuilder;
        use crate::chat::pipeline::model_strategy;

        Box::pin(async move {
            let system_prompt = ctx.prompt.render(self.prompt_kind(), session)?;

            let mut history = Vec::new();
            for m in &session.messages {
                history.extend(db_message_to_openrouter(ctx, &m.inner).await?);
            }

            let strategy = model_strategy::get_model_strategy(capability);
            let context_prompt = ctx.prompt.render_context(session)?;

            let messages = MessageBuilder::new(system_prompt)
                .history(history)
                .context(strategy.as_ref(), context_prompt)
                .build();

            // Get tools (async for MCP)
            let is_openrouter = !ctx.openrouter.is_compatibility_mode();
            let mut tools = ctx.tools.for_search_mode().await;

            if is_openrouter {
                tools.retain(|t| t.name != "web_search_tool" && t.name != "crawl_tool");
            }

            let strategy = model_strategy::get_model_strategy(capability);
            tools = strategy.filter_tools(tools);

            let options = openrouter::CompletionOption::builder()
                .web_search(true)
                .tools(&tools)
                .build();

            Ok(super::Execution::new(messages, options))
        })
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
                let output = state
                    .ctx
                    .tools
                    .execute_tool(&toolcall.name, &toolcall.args)
                    .await;

                state.session.message.inner.as_assistant().unwrap().push(
                    protocol::AssistantChunk::ToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    },
                );

                state
                    .session
                    .add_token(crate::chat::token::Token::ToolCall {
                        name: toolcall.name.clone(),
                        arg: toolcall.args.clone(),
                    });

                state
                    .messages
                    .push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                        id: toolcall.id.clone(),
                        name: toolcall.name.clone(),
                        arguments: toolcall.args.clone(),
                    }));

                // Emit rich content tokens to frontend (images, resources)
                for rich in &output.rich {
                    match rich {
                        crate::chat::tools::McpRichContent::Image { data, mime_type } => {
                            state.session.message.inner.as_assistant().unwrap().push(
                                protocol::AssistantChunk::McpImage {
                                    data: data.clone(),
                                    mime_type: mime_type.clone(),
                                },
                            );
                            state
                                .session
                                .add_token(crate::chat::token::Token::McpImage {
                                    data: data.clone(),
                                    mime_type: mime_type.clone(),
                                });
                        }
                        crate::chat::tools::McpRichContent::Resource {
                            uri,
                            mime_type,
                            text,
                        } => {
                            state.session.message.inner.as_assistant().unwrap().push(
                                protocol::AssistantChunk::McpResource {
                                    uri: uri.clone(),
                                    mime_type: mime_type.clone(),
                                    text: text.clone(),
                                },
                            );
                            state
                                .session
                                .add_token(crate::chat::token::Token::McpResource {
                                    uri: uri.clone(),
                                    mime_type: mime_type.clone(),
                                    text: text.clone(),
                                });
                        }
                    }
                }

                state.session.message.inner.as_assistant().unwrap().push(
                    protocol::AssistantChunk::ToolResult {
                        id: toolcall.id.clone(),
                        response: output.text.clone(),
                    },
                );

                state
                    .session
                    .add_token(crate::chat::token::Token::ToolResult(output.text.clone()));

                state.messages.push(openrouter::Message::ToolResult(
                    openrouter::MessageToolResult {
                        id: toolcall.id.clone(),
                        content: output.text,
                    },
                ));
            }

            Ok(false)
        })
    }
}
