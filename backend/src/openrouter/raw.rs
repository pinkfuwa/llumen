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
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<Plugin>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageReq>,
}

impl Default for CompletionReq {
    fn default() -> Self {
        Self {
            model: "openai/gpt-oss-20b".to_string(),
            messages: vec![],
            stream: true,
            tools: None,
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            plugins: Some(vec![Plugin {
                id: "file-parser".to_string(),
                pdf: PdfPlugin {
                    engine: "pdf-text".to_string(),
                },
            }]),
            usage: Some(UsageReq { include: true }),
        }
    }
}

impl CompletionReq {
    pub fn log(&self) {
        #[cfg(feature = "dev")]
        if let Ok(req) = serde_json::to_string_pretty(&self) {
            log::debug!(
                "sending completion\n===============\n{}\n===============",
                req
            );
        }
        #[cfg(not(feature = "dev"))]
        log::debug!("sending completion");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageReq {
    pub include: bool,
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

// TODO: have both content and contents set will cause serialization error
#[derive(Debug, Clone, Serialize, Default)]
pub struct Message {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallReq>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "content")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<InputAudio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<InputFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            "txt" | "md" | "json" | "csv" | "log" => {
                let content = String::from_utf8_lossy(&blob).to_string();

                (
                    Self::text(format!("Uploaded file: {}\n\n", filename)),
                    Self::text(content),
                )
            }
            _ => {
                // TODO: report unknown file type to user
                // Unknown file type is provider-specific, so provider may return error(we can't capture it)
                log::warn!("Unknown file type: {}", filename);
                (
                    Self::text(format!("Uploaded file: {}", filename)),
                    Self::file(filename.to_string(), blob),
                )
            }
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
                file_data: STANDARD.encode(file_data),
            }),
            ..Default::default()
        }
    }

    pub fn input_audio(data: Vec<u8>, format: String) -> Self {
        Self {
            r#type: MultiPartMessageType::InputAudio,
            input_audio: Some(InputAudio {
                data: STANDARD.encode(data),
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
    pub role: Option<Role>,
    pub content: Option<String>,
    pub reasoning: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCall {
    pub index: i64,
    pub id: Option<String>,
    pub function: ToolFunctionResp,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolFunctionResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FullChoice {
    pub index: i64,
    pub finish_reason: Option<FinishReason>,
    // logprobs aren't supported in most of providers
    pub logprobs: Option<f64>,
    pub message: OutputMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResponse {
    pub choices: Option<Vec<FullChoice>>,
    pub error: Option<ErrorInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputMessage {
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResp {
    pub error: ErrorInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorInfo {
    pub message: String,
    pub code: Option<i32>,
}
