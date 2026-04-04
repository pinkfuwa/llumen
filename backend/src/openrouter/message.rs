use protocol::OcrEngine;

use super::{error::Error, raw};
use crate::utils::blob::BlobReader;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::engine::Engine;

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub mime_type: String,
    pub data: BlobReader,
}

impl File {
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }
    pub fn is_pdf(&self) -> bool {
        self.mime_type == "application/pdf"
    }
    pub fn is_video(&self) -> bool {
        self.mime_type.starts_with("video/")
    }
    pub fn is_audio(&self) -> bool {
        self.mime_type.starts_with("audio/")
    }
}

/// Generated Image that haven't been stored
pub struct GeneratedImage {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl GeneratedImage {
    pub fn from_raw_image(raw: raw::Image) -> Result<Self, Error> {
        let raw::ImageUrl { url } = raw.image_url;
        let data_url = url
            .strip_prefix("data:")
            .ok_or_else(|| Error::Incompatible("Image URL missing data: prefix".into()))?;
        let (mime_part, base64_data) = data_url
            .split_once(';')
            .ok_or_else(|| Error::Incompatible("Image URL missing mime type".into()))?;
        let _mime_prefix = mime_part.strip_prefix("image/").unwrap_or(mime_part);
        let base64_data = base64_data
            .strip_prefix("base64,")
            .ok_or_else(|| Error::Incompatible("Image URL missing base64, prefix".into()))?;
        let data = BASE64_STANDARD
            .decode(base64_data)
            .map_err(|e| Error::Incompatible("Failed to decode base64 image"))?;
        Ok(Self {
            data,
            mime_type: mime_part.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MessageToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct MessageToolResult {
    pub id: String,
    pub content: String,
}

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
                let mut reasoning_details_value = None;
                if let Some(details) = reasoning_details {
                    if let Some(obj) = details.as_object() {
                        if let Some(model_id) = obj.get("model_id").and_then(|v| v.as_str()) {
                            if target_model_id.starts_with(model_id) {
                                reasoning_details_value = obj.get("data").cloned();
                            }
                        }
                    }
                }
                if files.is_empty() {
                    return raw::Message {
                        role: raw::Role::Assistant,
                        content: Some(content),
                        annotations,
                        reasoning_details: reasoning_details_value
                            .map(|v| vec![v])
                            .unwrap_or_default(),
                        ..Default::default()
                    };
                }
                let mut parts = Vec::new();

                for file in files {
                    todo!(
                        "read capability to see image support, and filter out image file if unsupport"
                    );
                }

                parts.push(raw::MessagePart::text(content));

                raw::Message {
                    role: raw::Role::Assistant,
                    contents: Some(parts),
                    annotations,
                    reasoning_details: reasoning_details_value.map(|v| vec![v]).unwrap_or_default(),
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
                    let (description, content) = raw::MessagePart::from_file(file);
                    parts.push(description);

                    // Filter based on content type and capabilities
                    match content.r#type {
                        raw::MultiPartMessageType::ImageUrl if capability.image_input => {
                            parts.push(content)
                        }
                        raw::MultiPartMessageType::InputAudio if capability.audio => {
                            parts.push(content)
                        }
                        raw::MultiPartMessageType::File
                            if capability.ocr != OcrEngine::Disabled =>
                        {
                            parts.push(content)
                        }
                        raw::MultiPartMessageType::Text => parts.push(content),
                        _ => {}
                    }
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
            Message::ToolResult(MessageToolResult { id, content }) => raw::Message {
                role: raw::Role::Tool,
                content: Some(content),
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
            structured_output: capability.json,
            toolcall: capability.tool,
            ocr: capability.ocr,
            audio: capability.audio,
            reasoning: capability.reasoning.map(|r| r.is_enabled()),
            reasoning_effort: capability.reasoning.map(|r| r.effort()),
        }
    }
}
