use protocol::OcrEngine;

use super::{SyncStream, error::Error, raw};

#[derive(Debug, Clone)]
pub struct File<S = Vec<u8>>
where
    S: SyncStream,
{
    pub name: String,
    pub data: S,
}

// generated image
#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl Image {
    /// Parse a data URL like "data:image/png;base64,iVBORw0KGgo..."
    pub fn from_data_url(url: &str) -> Result<Self, Error> {
        if !url.starts_with("data:") {
            return Err(Error::MalformedResponse(
                "Image URL does not start with 'data:'",
            ));
        }

        let url = url.strip_prefix("data:").unwrap();

        let parts: Vec<&str> = url.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(Error::MalformedResponse("Invalid data URL format"));
        }

        let metadata = parts[0];
        let base64_data = parts[1];

        // Parse metadata like "image/png;base64"
        let mime_type = if let Some(semicolon_pos) = metadata.find(';') {
            metadata[..semicolon_pos].to_string()
        } else {
            metadata.to_string()
        };

        // Decode base64
        let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_data)
            .map_err(|_| Error::MalformedResponse("Failed to decode base64 image data"))?;

        Ok(Image { data, mime_type })
    }

    pub fn from_raw_image(raw_image: super::raw::Image) -> Result<Self, Error> {
        Self::from_data_url(&raw_image.image_url.url)
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
pub enum Message<S = Vec<u8>>
where
    S: SyncStream,
{
    System(String),
    User(String),
    Assistant {
        content: String,
        annotations: Option<serde_json::Value>,
        reasoning_details: Option<serde_json::Value>,
        images: Vec<Image>,
    },
    MultipartUser {
        text: String,
        files: Vec<File<S>>,
    },
    ToolCall(MessageToolCall),
    ToolResult(MessageToolResult),
}

impl<S: SyncStream> Message<S> {
    /// Convert Message with streams to raw::Message and extract stream data
    /// Returns (raw::Message, Vec<(part_index, File<S>)>)
    pub fn to_raw_message_with_streams(
        self,
        target_model_id: &str,
        capability: &super::Capability,
    ) -> (raw::Message, Vec<(usize, File<S>)>) {
        match self {
            Message::Assistant {
                content,
                annotations,
                reasoning_details,
                images,
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
                if images.is_empty() {
                    return (
                        raw::Message {
                            role: raw::Role::Assistant,
                            content: Some(content),
                            annotations,
                            reasoning_details: reasoning_details_value
                                .map(|v| vec![v])
                                .unwrap_or_default(),
                            ..Default::default()
                        },
                        Vec::new(),
                    );
                }
                let mut parts = Vec::new();

                for image in images {
                    let data_url = format!(
                        "data:{};base64,{}",
                        image.mime_type,
                        base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &image.data
                        )
                    );
                    parts.push(raw::MessagePart::image_url(data_url));
                }

                parts.push(raw::MessagePart::text(content));

                (
                    raw::Message {
                        role: raw::Role::Assistant,
                        contents: Some(parts),
                        annotations,
                        reasoning_details: reasoning_details_value
                            .map(|v| vec![v])
                            .unwrap_or_default(),
                        ..Default::default()
                    },
                    Vec::new(),
                )
            }
            Message::System(msg) => (
                raw::Message {
                    role: raw::Role::System,
                    content: Some(msg),
                    ..Default::default()
                },
                Vec::new(),
            ),
            Message::User(msg) => (
                raw::Message {
                    role: raw::Role::User,
                    content: Some(msg),
                    ..Default::default()
                },
                Vec::new(),
            ),

            Message::MultipartUser { text, files } => {
                let mut parts = vec![raw::MessagePart::text(text)];
                let mut stream_files = Vec::new();

                for file in files {
                    let part_index = parts.len();
                    let (description, content) = raw::MessagePart::unknown_placeholder(&file.name);
                    parts.push(description);

                    // Filter based on content type and capabilities
                    let should_include = match content.r#type {
                        raw::MultiPartMessageType::ImageUrl => capability.image_input,
                        raw::MultiPartMessageType::InputAudio => capability.audio,
                        raw::MultiPartMessageType::File => capability.ocr != OcrEngine::Disabled,
                        raw::MultiPartMessageType::Text => true,
                    };

                    if should_include {
                        parts.push(content);
                        stream_files.push((part_index + 1, file));
                    }
                }

                (
                    raw::Message {
                        role: raw::Role::User,
                        contents: Some(parts),
                        ..Default::default()
                    },
                    stream_files,
                )
            }
            Message::ToolCall(MessageToolCall {
                id,
                name,
                arguments,
            }) => (
                raw::Message {
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
                Vec::new(),
            ),
            Message::ToolResult(MessageToolResult { id, content }) => (
                raw::Message {
                    role: raw::Role::Tool,
                    content: Some(content),
                    tool_call_id: Some(id),
                    ..Default::default()
                },
                Vec::new(),
            ),
        }
    }

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
                images,
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
                if images.is_empty() {
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

                for image in images {
                    let data_url = format!(
                        "data:{};base64,{}",
                        image.mime_type,
                        base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &image.data
                        )
                    );
                    parts.push(raw::MessagePart::image_url(data_url));
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
                // This path should only be used with Vec<u8> where we can inline the data
                let mut parts = vec![raw::MessagePart::text(text)];

                for file in files {
                    let mut data_vec = Vec::new();
                    let mut file_data = file.data;
                    file_data.read(&mut data_vec);

                    let (description, content) = raw::MessagePart::unknown(&file.name, data_vec);
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
            image_output: capability.image,
            image_input: None,
            structured_output: capability.json,
            toolcall: capability.tool,
            ocr: capability.ocr,
            audio: capability.audio,
            reasoning: capability.reasoning,
        }
    }
}
