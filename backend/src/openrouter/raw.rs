//! OpenRouter API request and response structures
//!
//! Not codegen, but it match the API spec
//!
//! https://openrouter.ai/docs
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CompletionReq {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    pub plugins: Plugin,
}

impl Default for CompletionReq {
    fn default() -> Self {
        Self {
            model: "openai/gpt-oss-20b:free".to_string(),
            messages: vec![],
            stream: true,
            tools: None,
            plugins: Plugin {
                id: "file-parser".to_string(),
                pdf: PdfPlugin {
                    engine: "pdf-text".to_string(),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Plugin {
    pub id: String,
    pub pdf: PdfPlugin,
}

#[derive(Debug, Clone, Serialize)]
pub struct PdfPlugin {
    pub engine: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub r#type: String,
    pub function: FunctionTool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallReq>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<Vec<MessagePart>>,
}

// `data:image/jpeg;base64,${base64Image}`;
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MultiPartMessageType {
    #[default]
    Text,
    ImageUrl,
    File,
    InputAudio,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MessagePart {
    pub r#type: MultiPartMessageType,
    pub text: Option<String>,
    pub input_audio: Option<InputAudio>,
    pub file: Option<InputFile>,
    pub image_url: Option<InputImage>,
}

impl MessagePart {
    pub fn text(text: String) -> Self {
        Self {
            r#type: MultiPartMessageType::Text,
            text: Some(text),
            ..Default::default()
        }
    }

    pub fn unknown(filename: &str, blob: Vec<u8>) -> (Self, Self) {
        let ext = filename
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::image_url(format!(
                    "data:image/{};base64,{}",
                    ext,
                    STANDARD.encode(&blob)
                )),
            ),
            "mp3" | "wav" | "flac" | "m4a" => (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::input_audio(blob, ext),
            ),
            _ => (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::file(filename.to_string(), blob),
            ),
        }
    }

    pub fn image_url(url: String) -> Self {
        Self {
            r#type: MultiPartMessageType::ImageUrl,
            image_url: Some(InputImage { url }),
            ..Default::default()
        }
    }

    pub fn file(filename: String, file_data: Vec<u8>) -> Self {
        Self {
            r#type: MultiPartMessageType::File,
            file: Some(InputFile {
                filename,
                file_data: base64::encode(file_data),
            }),
            ..Default::default()
        }
    }

    pub fn input_audio(data: Vec<u8>, format: String) -> Self {
        Self {
            r#type: MultiPartMessageType::InputAudio,
            input_audio: Some(InputAudio {
                data: base64::encode(data),
                format,
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InputAudio {
    data: String,
    format: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputFile {
    filename: String,
    file_data: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputImage {
    url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallReq {
    pub id: String,
    pub function: ToolFunctionResp,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResp {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
}

/// openrouter specific response with usage and cost info
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionInfoResp {
    pub id: String,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub total_tokens: Option<i64>,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i64,
    pub delta: Delta,
    pub finish_reason: Option<FinishReason>,
    // logprobs aren't supported in most of providers
    pub logprobs: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    pub role: Role,
    pub content: String,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCall {
    pub index: i64,
    pub id: String,
    pub function: ToolFunctionResp,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolFunctionResp {
    pub arguments: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResponse {
    pub output: Vec<OutputMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputMessage {
    pub r#type: String,
    pub role: String,
    pub content: Vec<OutputContent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputContent {
    pub r#type: Option<String>,
    pub text: Option<String>,
}
