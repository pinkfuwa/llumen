use std::sync::Arc;

use super::configuration::Configuration;
use super::executor::execute_search_tool;
use crate::{chat::*, openrouter};

pub fn search_configuration() -> Configuration {
    use crate::chat::tools::{get_crawl_tool_def, get_web_search_tool_def};

    Configuration {
        completion_option: openrouter::CompletionOption::builder()
            .web_search(true)
            .tools(&[get_web_search_tool_def(), get_crawl_tool_def()])
            .build(),
        tool_handler: Arc::new(|state, toolcalls| {
            Box::pin(async move {
                let assistant_text = state
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
                    state.messages.push(openrouter::Message::Assistant {
                        content: assistant_text,
                        annotations: None,
                        reasoning_details: None,
                    });
                }

                for toolcall in toolcalls {
                    let result =
                        execute_search_tool(&state.ctx, &toolcall.name, &toolcall.args).await;

                    state
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

                    state
                        .completion_ctx
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
                        .completion_ctx
                        .message
                        .inner
                        .as_assistant()
                        .unwrap()
                        .push(protocol::AssistantChunk::ToolResult {
                            id: toolcall.id.clone(),
                            response: result.clone(),
                        });

                    state
                        .completion_ctx
                        .add_token(crate::chat::token::Token::ToolResult(result.clone()));

                    state.messages.push(openrouter::Message::ToolResult(
                        openrouter::MessageToolResult {
                            id: toolcall.id.clone(),
                            content: result,
                        },
                    ));
                }

                Ok(false)
            })
        }),
        prompt: prompt::PromptKind::Search,
    }
}
