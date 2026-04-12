use anyhow::Result;
use protocol::AssistantChunk;
use tokio_stream::StreamExt;

use crate::chat::context::StreamEndReason;
use crate::chat::converter::{openrouter_stream_to_assitant_chunk, openrouter_to_buffer_token};
use crate::chat::session::CompletionSession;
use crate::chat::token::Token;
use crate::chat::Context;
use crate::openrouter::{self, AspectRatio, MessageToolResult, StreamWithOrderedTokens};

#[derive(serde::Deserialize)]
struct GenerateImageArgs {
    prompt: String,
    aspect_ratio: String,
    #[serde(default)]
    reference_file_names: Vec<String>,
}

fn parse_aspect_ratio(value: &str) -> Option<AspectRatio> {
    match value {
        "1:1" => Some(AspectRatio::R1x1),
        "2:3" => Some(AspectRatio::R2x3),
        "3:2" => Some(AspectRatio::R3x2),
        "3:4" => Some(AspectRatio::R3x4),
        "4:3" => Some(AspectRatio::R4x3),
        "4:5" => Some(AspectRatio::R4x5),
        "5:4" => Some(AspectRatio::R5x4),
        "9:16" => Some(AspectRatio::R9x16),
        "16:9" => Some(AspectRatio::R16x9),
        "21:9" => Some(AspectRatio::R21x9),
        _ => None,
    }
}

pub async fn execute(ctx: &Context, session: &mut CompletionSession) -> Result<()> {
    if session.model.config.media_gen.image_model.is_none()
        && session.model.config.media_gen.video_model.is_none()
    {
        session.add_error("Media generation is disabled for this model config.".to_string());
        return Ok(());
    }

    let tools = ctx.tools.for_media_mode();
    let option = openrouter::CompletionOption::builder()
        .tools(&tools)
        .build();
    let mut messages = session.assemble_messages(ctx, option.clone())?;

    loop {
        let model = session.openrouter_model();
        let stream: openrouter::StreamCompletion = ctx
            .openrouter
            .stream(model, messages.clone(), option.clone())
            .await?;

        let mut ordered_stream = StreamWithOrderedTokens::new(stream);

        let halt = session
            .put_stream((&mut ordered_stream).map(|resp| resp.map(openrouter_to_buffer_token)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            return Ok(());
        }

        let stream = ordered_stream.into_inner();
        let mut result = stream.get_result();
        session.update_usage(result.usage.cost as f32, result.usage.token as i32);
        session.apply_stream_result(&result).await;

        let tool_calls = std::mem::take(&mut result.toolcalls);
        let assistant_text = result.get_text();

        let chunks = openrouter_stream_to_assitant_chunk(&result.responses);
        session.extend_chunks(chunks);

        if tool_calls.is_empty() {
            break;
        }

        for tool_call in &tool_calls {
            session.add_token(Token::ToolCall {
                name: tool_call.name.clone(),
                arg: tool_call.args.clone(),
            });
        }

        messages.push(openrouter::Message::Assistant {
            content: assistant_text,
            annotations: None,
            reasoning_details: None,
            files: Vec::new(),
        });

        for tool_call in tool_calls {
            messages.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                arguments: tool_call.args.clone(),
            }));

            session.add_chunk(AssistantChunk::ToolCall {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                arg: tool_call.args.clone(),
            });

            let (content, files) =
                execute_tool(ctx, session, &tool_call.name, &tool_call.args).await;

            messages.push(openrouter::Message::ToolResult(MessageToolResult {
                id: tool_call.id.clone(),
                content: content.clone(),
                files: files.clone(),
            }));

            session.add_token(Token::ToolResult {
                content: content.clone(),
                files: files.clone(),
            });

            session.add_chunk(AssistantChunk::ToolResult {
                id: tool_call.id,
                response: content,
                files,
            });
        }
    }

    Ok(())
}

async fn execute_tool(
    ctx: &Context,
    session: &mut CompletionSession,
    name: &str,
    args: &str,
) -> (String, Vec<protocol::FileMetadata>) {
    if name != "generate_image" {
        return (format!("Unknown tool: {name}"), Vec::new());
    }

    let parsed: GenerateImageArgs = match serde_json::from_str(args) {
        Ok(value) => value,
        Err(error) => {
            return (
                format!("Invalid arguments for generate_image: {error}"),
                Vec::new(),
            );
        }
    };

    let aspect_ratio = match parse_aspect_ratio(&parsed.aspect_ratio) {
        Some(value) => value,
        None => {
            return (
                format!("Unsupported aspect_ratio: {}", parsed.aspect_ratio),
                Vec::new(),
            );
        }
    };

    let image_model = match session.model.config.media_gen.image_model.clone() {
        Some(model) => model,
        None => {
            return (
                "Model config missing [media_gen].image_model".to_string(),
                Vec::new(),
            );
        }
    };

    let mut reference_images = Vec::new();
    if !parsed.reference_file_names.is_empty() {
        let user_files = session
            .history
            .iter()
            .rev()
            .find_map(|message| match &message.inner {
                protocol::MessageInner::User { files, .. } => {
                    if files.is_empty() {
                        None
                    } else {
                        Some(files.clone())
                    }
                }
                _ => None,
            })
            .unwrap_or_default();

        for reference_name in &parsed.reference_file_names {
            if let Some(file_meta) = user_files.iter().find(|file| file.name == *reference_name) {
                if let Some(reader) = ctx.blob.get(file_meta.id) {
                    reference_images.push(openrouter::File {
                        name: file_meta.name.clone(),
                        data: reader.into(),
                    });
                }
            }
        }
    }

    let output = match ctx
        .openrouter
        .image_generate(
            image_model,
            parsed.prompt.clone(),
            reference_images,
            aspect_ratio,
        )
        .await
    {
        Ok(output) => output,
        Err(error) => {
            return (format!("Image generation failed: {error}"), Vec::new());
        }
    };

    session.update_usage(output.price as f32, output.token as i32);

    let mut file_refs = Vec::new();
    for image in &output.images {
        if let Ok(file_id) = session.store_blob_file(image).await {
            file_refs.push(protocol::FileMetadata {
                id: file_id,
                name: format!("generated-image-{file_id}.png"),
            });
            session.add_token(Token::Image(file_id));
        }
    }

    let summary = match output.text {
        Some(text) if !text.trim().is_empty() => text,
        _ => "Generated image successfully.".to_string(),
    };

    (summary, file_refs)
}
