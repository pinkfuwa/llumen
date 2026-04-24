use anyhow::Result;
use protocol::{AssistantChunk, FileMetadata};
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
    generating_file: Option<String>,
    #[serde(default)]
    reference_files: Vec<String>,
}

#[derive(serde::Deserialize)]
struct GenerateVideoArgs {
    prompt: String,
    #[serde(default)]
    generating_file: Option<String>,
    #[serde(default)]
    duration: Option<u32>,
    #[serde(default)]
    resolution: Option<String>,
    #[serde(default)]
    aspect_ratio: Option<String>,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    generate_audio: Option<bool>,
    #[serde(default)]
    reference_files: Vec<String>,
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

    let tools = ctx.tools.for_media_mode(&session.model.config.media_gen);
    let option = openrouter::CompletionOption::builder()
        .tools(&tools)
        .session_id(session.chat.id.to_string())
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
    match name {
        "generate_image" => execute_generate_image(ctx, session, args).await,
        "generate_video" => execute_generate_video(ctx, session, args).await,
        _ => (format!("Unknown tool: {name}"), Vec::new()),
    }
}

fn latest_reference_files(session: &CompletionSession) -> Vec<FileMetadata> {
    let mut reference_files = Vec::new();

    session
        .history
        .iter()
        .for_each(|message| match &message.inner {
            protocol::MessageInner::User { files, .. } => {
                reference_files.extend(files.clone());
            }
            protocol::MessageInner::Assistant(chunks) => {
                for chunk in chunks {
                    if let AssistantChunk::ToolResult { files, .. } = chunk {
                        reference_files.extend(files.clone());
                    }
                }
            }
        });

    reference_files
}

fn resolve_reference_files(
    session: &CompletionSession,
    reference_file_names: &[String],
) -> Vec<FileMetadata> {
    if reference_file_names.is_empty() {
        return Vec::new();
    }

    let reference_files = latest_reference_files(session);
    reference_file_names
        .iter()
        .filter_map(|reference_name| {
            reference_files
                .iter()
                .find(|file| file.name == *reference_name)
                .cloned()
        })
        .collect()
}

fn generated_file_name(file_name: Option<String>, default_name: &str) -> String {
    file_name.unwrap_or_else(|| default_name.to_string())
}

async fn execute_generate_image(
    ctx: &Context,
    session: &mut CompletionSession,
    args: &str,
) -> (String, Vec<protocol::FileMetadata>) {
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

    let resolved_files = resolve_reference_files(session, &parsed.reference_files);
    let mut missing_files = Vec::new();
    let reference_images = resolved_files
        .into_iter()
        .filter_map(|file_meta| match ctx.blob.get(file_meta.id) {
            Some(reader) => Some(openrouter::File {
                name: file_meta.name,
                data: reader.into(),
                mime_type: session.file_mime_type(file_meta.id).map(str::to_string),
            }),
            None => {
                missing_files.push(file_meta.name);
                None
            }
        })
        .collect::<Vec<_>>();

    if !missing_files.is_empty() {
        let names = missing_files.join(", ");
        return (
            format!("Referenced image(s) not found: {names}"),
            Vec::new(),
        );
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
            let file_name = generated_file_name(
                parsed.generating_file.clone(),
                &format!("generated-image-{file_id}.png"),
            );
            file_refs.push(protocol::FileMetadata {
                id: file_id,
                name: file_name,
                kind: protocol::FileKind::Image,
            });
            // TODO: session.add_token(Token::Image(file_id));
        }
    }

    let summary = if file_refs.is_empty() {
        match output.text {
            Some(text) if !text.trim().is_empty() => text,
            _ => "Generated image successfully.".to_string(),
        }
    } else {
        let names = file_refs
            .iter()
            .map(|file| file.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Generated image successfully: {names}.")
    };

    (summary, file_refs)
}

async fn execute_generate_video(
    ctx: &Context,
    session: &mut CompletionSession,
    args: &str,
) -> (String, Vec<protocol::FileMetadata>) {
    let parsed: GenerateVideoArgs = match serde_json::from_str(args) {
        Ok(value) => value,
        Err(error) => {
            return (
                format!("Invalid arguments for generate_video: {error}"),
                Vec::new(),
            );
        }
    };

    let video_model = match session.model.config.media_gen.video_model.clone() {
        Some(model) => model,
        None => {
            return (
                "Model config missing [media_gen].video_model".to_string(),
                Vec::new(),
            );
        }
    };

    let resolved_files = resolve_reference_files(session, &parsed.reference_files);
    let mut missing_files = Vec::new();
    let references = resolved_files
        .into_iter()
        .filter_map(|file_meta| match ctx.blob.get(file_meta.id) {
            Some(reader) => Some(openrouter::File {
                name: file_meta.name,
                data: reader.into(),
                mime_type: session.file_mime_type(file_meta.id).map(str::to_string),
            }),
            None => {
                missing_files.push(file_meta.name);
                None
            }
        })
        .collect::<Vec<_>>();

    if !missing_files.is_empty() {
        let names = missing_files.join(", ");
        return (format!("Referenced file(s) not found: {names}"), Vec::new());
    }

    let option = openrouter::VideoGenerationOption {
        duration: parsed.duration,
        resolution: parsed.resolution,
        aspect_ratio: parsed.aspect_ratio,
        size: parsed.size,
        generate_audio: parsed.generate_audio,
        ..Default::default()
    };

    let mut output = match ctx
        .openrouter
        .video_generate(video_model, parsed.prompt.clone(), references, option)
        .await
    {
        Ok(output) => output,
        Err(error) => {
            return (format!("Video generation failed: {error}"), Vec::new());
        }
    };

    session.update_usage(output.price as f32, 0);

    let mut file_refs = Vec::new();
    for video in &mut output.videos {
        if let Ok(file_id) = session.store_blob_video(video).await {
            let file_name = generated_file_name(
                parsed.generating_file.clone(),
                &format!("generated-video-{file_id}.mp4"),
            );
            file_refs.push(protocol::FileMetadata {
                id: file_id,
                name: file_name,
                kind: protocol::FileKind::Video,
            });
        }
    }

    let summary = if file_refs.is_empty() {
        format!("Video generation completed (job {}).", output.job_id)
    } else {
        let names = file_refs
            .iter()
            .map(|file| file.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Generated video successfully (job {}): {names}.",
            output.job_id
        )
    };

    (summary, file_refs)
}
