//! Application-level message types and conversion.

use protocol::OcrEngine;

use super::{error::Error, raw};
use crate::utils::blob::BlobReader;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::engine::Engine;

/// A file attached to a message, containing the file name, binary data, and
/// optional MIME type.
#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub data: BlobReader,
    pub mime_type: Option<String>,
}

fn file_to_parts(
    file: File,
    capability: &super::Capability,
) -> impl Iterator<Item = raw::MessagePart> + '_ {
    raw::MessagePart::from_file(file)
        .into_iter()
        .filter(move |part| match part.r#type {
            raw::MultiPartMessageType::ImageUrl => capability.image_input,
            raw::MultiPartMessageType::InputAudio => capability.audio,
            raw::MultiPartMessageType::VideoUrl => capability.video_input,
            raw::MultiPartMessageType::File => capability.ocr != OcrEngine::Disabled,
            raw::MultiPartMessageType::Text => true,
        })
}

/// Generated Image that haven't been stored
pub struct GeneratedImage {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl GeneratedImage {
    /// Decodes a base64-encoded JSON string into a [`GeneratedImage`] with the
    /// given MIME type.
    pub fn from_b64_json(b64_json: String, mime_type: impl Into<String>) -> Result<Self, Error> {
        let data = BASE64_STANDARD
            .decode(&b64_json)
            .map_err(|_| Error::Incompatible("Failed to decode base64 image"))?;
        Ok(Self {
            data,
            mime_type: mime_type.into(),
        })
    }

    /// Extracts and decodes image data from a [`raw::Image`] data URL into a
    /// [`GeneratedImage`].
    pub fn from_raw_image(raw: raw::Image) -> Result<Self, Error> {
        let raw::ImageUrl { url } = raw.image_url;
        let data_url = url
            .strip_prefix("data:")
            .ok_or_else(|| Error::Incompatible("Image URL missing data: prefix"))?;
        let (mime_part, base64_data) = data_url
            .split_once(';')
            .ok_or_else(|| Error::Incompatible("Image URL missing mime type"))?;
        let base64_data = base64_data
            .strip_prefix("base64,")
            .ok_or_else(|| Error::Incompatible("Image URL missing base64, prefix"))?;
        let data = BASE64_STANDARD
            .decode(base64_data)
            .map_err(|_| Error::Incompatible("Failed to decode base64 image"))?;
        Ok(Self {
            data,
            mime_type: mime_part.to_string(),
        })
    }
}

/// A tool invocation returned by the assistant, containing the call ID,
/// function name, and arguments.
#[derive(Debug, Clone)]
pub struct MessageToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// The result of a tool call, including the call ID, text content, and
/// associated file metadata.
#[derive(Debug, Clone)]
pub struct MessageToolResult {
    pub id: String,
    pub content: String,
    pub files: Vec<protocol::FileMetadata>,
}

/// Application-level message variants: system, user, assistant, multipart user,
/// tool call, and tool result.
#[derive(Debug, Clone)]
pub enum Message {
    System(String),
    User(String),
    Assistant {
        content: String,
        annotations: Option<serde_json::Value>,
        reasoning_details: Option<serde_json::Value>,
        files: Vec<File>,
    },
    MultipartUser {
        text: String,
        files: Vec<File>,
    },
    ToolCall(MessageToolCall),
    ToolResult(MessageToolResult),
}

impl Message {
    /// Converts this application-level message into the protocol-level
    /// [`raw::Message`], filtering file parts based on the model's
    /// capabilities.
    pub fn to_raw_message(
        self,
        target_model_id: &str,
        capability: &super::Capability,
    ) -> raw::Message {
        match self {
            Message::Assistant {
                content,
                annotations,
                reasoning_details,
                files,
            } => {
                let reasoning_details = reasoning_details
                    .and_then(|details| {
                        let stored_model_id = details.get("model_id")?.as_str()?;
                        let stored_model_id = stored_model_id.split(':').next()?;
                        let target_model_id = target_model_id.split(':').next()?;
                        if stored_model_id.is_empty() || stored_model_id != target_model_id {
                            return None;
                        }
                        details.get("data")?.as_array().cloned()
                    })
                    .unwrap_or_default();
                if files.is_empty() {
                    return raw::Message {
                        role: raw::Role::Assistant,
                        content: Some(content),
                        annotations,
                        reasoning_details,
                        ..Default::default()
                    };
                }
                let mut parts = Vec::new();

                for file in files {
                    parts.extend(file_to_parts(file, capability));
                }

                parts.push(raw::MessagePart::text(content));

                raw::Message {
                    role: raw::Role::Assistant,
                    contents: Some(parts),
                    annotations,
                    reasoning_details,
                    ..Default::default()
                }
            }
            Message::System(msg) => raw::Message {
                role: raw::Role::System,
                content: Some(msg),
                ..Default::default()
            },
            Message::User(msg) => raw::Message {
                role: raw::Role::User,
                content: Some(msg),
                ..Default::default()
            },

            Message::MultipartUser { text, files } => {
                let mut parts = vec![raw::MessagePart::text(text)];

                for file in files {
                    parts.extend(file_to_parts(file, capability));
                }

                raw::Message {
                    role: raw::Role::User,
                    contents: Some(parts),
                    ..Default::default()
                }
            }
            Message::ToolCall(MessageToolCall {
                id,
                name,
                arguments,
            }) => raw::Message {
                role: raw::Role::Assistant,
                tool_calls: Some(vec![raw::ToolCallReq {
                    id,
                    function: raw::ToolFunctionResp {
                        name: Some(name),
                        arguments: Some(arguments),
                    },
                    r#type: "function".to_string(),
                }]),
                content: Some("".to_string()),
                ..Default::default()
            },
            Message::ToolResult(MessageToolResult { id, content, files }) => raw::Message {
                role: raw::Role::Tool,
                content: Some(match files.is_empty() {
                    true => content,
                    false => {
                        let payload = serde_json::json!({
                            "content": content,
                            "files": files,
                        });
                        payload.to_string()
                    }
                }),
                tool_call_id: Some(id),
                ..Default::default()
            },
        }
    }
}

impl From<protocol::ModelCapability> for super::MaybeCapability {
    fn from(capability: protocol::ModelCapability) -> Self {
        super::MaybeCapability {
            text_output: None,
            image_output: capability.image,
            image_input: None,
            video_input: capability.video,
            structured_output: capability.json,
            toolcall: capability.tool,
            ocr: capability.ocr,
            audio: capability.audio,
            reasoning: capability.reasoning.map(|r| r.is_enabled()),
            reasoning_effort: capability.reasoning.map(|r| r.effort()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use stream_json::IntoSerializer;

    fn assistant(reasoning_details: Option<Value>) -> Message {
        Message::Assistant {
            content: "Answer 世界".to_string(),
            annotations: None,
            reasoning_details,
            files: Vec::new(),
        }
    }

    fn reasoning_blocks() -> Value {
        json!([
            {"type": "reasoning.text", "text": "Think", "index": 0},
            {"type": "reasoning.encrypted", "data": "opaque", "index": 1}
        ])
    }

    async fn serialize(message: raw::Message) -> anyhow::Result<Value> {
        let body = reqwest::Body::wrap_stream(message.into_stream());
        let bytes = axum::body::to_bytes(axum::body::Body::new(body), usize::MAX).await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    #[tokio::test]
    async fn serializes_system_and_user_roles() -> anyhow::Result<()> {
        for (message, role) in [
            (Message::System("Guide".to_string()), "system"),
            (Message::User("Question".to_string()), "user"),
        ] {
            let content = match &message {
                Message::System(content) | Message::User(content) => content.clone(),
                _ => unreachable!(),
            };
            let message = message.to_raw_message("test/model", &Default::default());
            assert_eq!(
                serialize(message).await?,
                json!({"role": role, "content": content})
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn preserves_assistant_annotations_without_reasoning() -> anyhow::Result<()> {
        let annotations =
            json!([{"type": "url_citation", "url_citation": {"url": "https://example.com"}}]);
        let mut message = assistant(None);
        if let Message::Assistant {
            annotations: value, ..
        } = &mut message
        {
            *value = Some(annotations.clone());
        }
        let message = message.to_raw_message("test/model", &Default::default());
        assert_eq!(
            serialize(message).await?,
            json!({
                "role": "assistant", "content": "Answer 世界", "annotations": annotations
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn replays_reasoning_as_a_flat_array_in_original_order() -> anyhow::Result<()> {
        let blocks = reasoning_blocks();
        let message = assistant(Some(json!({"model_id": "test/model", "data": blocks})))
            .to_raw_message("test/model", &Default::default());
        assert_eq!(
            serialize(message).await?,
            json!({
                "role": "assistant", "content": "Answer 世界", "reasoning_details": blocks
            })
        );
        Ok(())
    }

    #[test]
    fn preserves_reasoning_across_routing_suffixes() {
        for (stored, target) in [
            ("test/model", "test/model:free"),
            ("test/model:free", "test/model"),
            ("test/model:free", "test/model:nitro"),
        ] {
            let blocks = reasoning_blocks();
            let message = assistant(Some(json!({"model_id": stored, "data": blocks})))
                .to_raw_message(target, &Default::default());
            assert_eq!(
                message.reasoning_details,
                blocks.as_array().cloned().unwrap_or_default()
            );
        }
    }

    #[test]
    fn omits_reasoning_for_other_models_including_prefix_collisions() {
        for target in ["other/model", "test/model-v2", "test/model-extra:free"] {
            let message = assistant(Some(json!({
                "model_id": "test/model", "data": reasoning_blocks()
            })))
            .to_raw_message(target, &Default::default());
            assert!(message.reasoning_details.is_empty(), "target: {target}");
            assert_eq!(message.content.as_deref(), Some("Answer 世界"));
        }
    }

    #[test]
    fn omits_malformed_stored_reasoning() {
        for details in [
            Value::Null,
            json!([]),
            json!({"data": reasoning_blocks()}),
            json!({"model_id": 42, "data": reasoning_blocks()}),
            json!({"model_id": "", "data": reasoning_blocks()}),
            json!({"model_id": "test/model"}),
            json!({"model_id": "test/model", "data": null}),
            json!({"model_id": "test/model", "data": "invalid"}),
            json!({"model_id": "test/model", "data": {"text": "invalid"}}),
        ] {
            let message =
                assistant(Some(details.clone())).to_raw_message("test/model", &Default::default());
            assert!(message.reasoning_details.is_empty(), "details: {details}");
            assert_eq!(message.content.as_deref(), Some("Answer 世界"));
        }
    }

    #[tokio::test]
    async fn omits_empty_reasoning_from_the_request() -> anyhow::Result<()> {
        let message = assistant(Some(json!({"model_id": "test/model", "data": []})))
            .to_raw_message("test/model", &Default::default());
        assert_eq!(
            serialize(message).await?,
            json!({"role": "assistant", "content": "Answer 世界"})
        );
        Ok(())
    }

    #[tokio::test]
    async fn preserves_reasoning_and_annotations_in_multipart_assistant() -> anyhow::Result<()> {
        let database =
            redb::Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?;
        let blob = crate::utils::blob::BlobDB::new(std::sync::Arc::new(database));
        blob.insert(
            1,
            11,
            tokio_stream::iter([bytes::Bytes::from_static(b"source text")]),
        )
        .await?;
        let reader = blob
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("test blob missing"))?;
        let blocks = reasoning_blocks();
        let annotations = json!([{"type": "file", "file": {"name": "source.txt"}}]);
        let message = Message::Assistant {
            content: "Answer".to_string(),
            annotations: Some(annotations.clone()),
            reasoning_details: Some(json!({"model_id": "test/model", "data": blocks})),
            files: vec![File {
                name: "source.txt".to_string(),
                data: reader.into(),
                mime_type: Some("text/plain".to_string()),
            }],
        }
        .to_raw_message("test/model", &Default::default());
        assert_eq!(
            serialize(message).await?,
            json!({
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "\nUploaded file: source.txt\n<content>\n"},
                    {"type": "text", "text": "source text"},
                    {"type": "text", "text": "\n</content>"},
                    {"type": "text", "text": "Answer"}
                ],
                "annotations": annotations,
                "reasoning_details": blocks
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn serializes_tool_calls_with_original_arguments() -> anyhow::Result<()> {
        let arguments = "{\"query\": \"世界\"}";
        let message = Message::ToolCall(MessageToolCall {
            id: "call_1".to_string(),
            name: "search".to_string(),
            arguments: arguments.to_string(),
        })
        .to_raw_message("test/model", &Default::default());
        assert_eq!(
            serialize(message).await?,
            json!({
                "role": "assistant", "content": "",
                "tool_calls": [{"id": "call_1", "type": "function",
                    "function": {"name": "search", "arguments": arguments}}]
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn serializes_tool_results_with_optional_file_metadata() -> anyhow::Result<()> {
        for files in [
            Vec::new(),
            vec![protocol::FileMetadata {
                id: 7,
                name: "result.txt".to_string(),
                kind: Default::default(),
                dimensions: None,
            }],
        ] {
            let content = if files.is_empty() {
                "Found 世界".to_string()
            } else {
                json!({"content": "Found 世界", "files": files}).to_string()
            };
            let message = Message::ToolResult(MessageToolResult {
                id: "call_1".to_string(),
                content: "Found 世界".to_string(),
                files,
            })
            .to_raw_message("test/model", &Default::default());
            assert_eq!(
                serialize(message).await?,
                json!({
                    "role": "tool", "tool_call_id": "call_1", "content": content
                })
            );
        }
        Ok(())
    }
}
