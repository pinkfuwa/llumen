//! OpenRouter API request and response structures
//!
//! Not codegen, but it match the API spec
//!
//! https://openrouter.ai/docs
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize)]
pub struct ModelListResponse {
    pub data: Vec<Model>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// Native image(can still input file without it)
    File,
    Image,
    Text,
    Audio,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportedParams {
    ResponseFormat,
    Tools,
    StructuredOutput,
    Reasoning,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub id: String,
    pub name: Option<String>,
    #[serde(default)]
    pub supported_parameters: Vec<SupportedParams>,
    #[serde(default)]
    pub architecture: Architecture,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Architecture {
    #[serde(default)]
    pub input_modalities: Vec<Modality>,
    #[serde(default)]
    pub output_modalities: Vec<Modality>,
}

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
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<Plugin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageReq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    // reasoning options
    #[serde(skip_serializing_if = "Reasoning::is_empty")]
    pub reasoning: Reasoning,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modalities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Reasoning {
    pub effort: Option<String>,
    pub enabled: Option<bool>,
}

impl Reasoning {
    pub fn is_empty(&self) -> bool {
        self.effort.is_none() && self.enabled.is_none()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseFormat {
    pub r#type: String,
    pub json_schema: serde_json::Value,
}
// "response_format": {
//   "type": "json_schema",
//   "json_schema": {
//     "name": "weather",
//     "strict": true,
//     "schema": {
//       "type": "object",
//       "properties": {
//         "location": {
//           "type": "string",
//           "description": "City or location name"
//         },
//         "temperature": {
//           "type": "number",
//           "description": "Temperature in Celsius"
//         },
//         "conditions": {
//           "type": "string",
//           "description": "Weather conditions description"
//         }
//       },
//       "required": ["location", "temperature", "conditions"],
//       "additionalProperties": false
//     }
//   }
// }

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf: Option<PdfPlugin>,
}

impl Plugin {
    pub fn pdf_text() -> Self {
        Self {
            id: "file-parser".to_string(),
            pdf: Some(PdfPlugin {
                engine: "pdf-text".to_string(),
            }),
        }
    }
    pub fn mistral_ocr() -> Self {
        Self {
            id: "file-parser".to_string(),
            pdf: Some(PdfPlugin {
                engine: "mistral-ocr".to_string(),
            }),
        }
    }
    pub fn pdf_native() -> Self {
        Self {
            id: "file-parser".to_string(),
            pdf: Some(PdfPlugin {
                engine: "native".to_string(),
            }),
        }
    }
    pub fn web() -> Self {
        Self {
            id: "web".to_string(),
            pdf: None,
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,
    // reasoning text or encrypted reasoning detail
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasoning_details: Vec<serde_json::Value>,
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
        infer::is_image(&blob);

        let ext = filename
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if infer::is_image(&blob) {
            return (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::image_url(format!(
                    "data:image/{};base64,{}",
                    ext,
                    STANDARD.encode(&blob)
                )),
            );
        }

        if infer::is_audio(&blob) {
            return (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::input_audio(blob, ext),
            );
        }

        if infer::archive::is_pdf(&blob)
            || infer::archive::is_epub(&blob)
            || infer::is_document(&blob)
        {
            return (
                Self::text(format!("Uploaded file: {}", filename)),
                Self::pdf(filename.to_string(), &blob),
            );
        }

        let content = String::from_utf8_lossy(&blob);

        (
            Self::text(format!("Uploaded file: {}\n\n", filename)),
            Self::text(format!("<content>{}</content>", content)),
        )
    }

    pub fn image_url(url: String) -> Self {
        Self {
            r#type: MultiPartMessageType::ImageUrl,
            image_url: Some(InputImage { url }),
            ..Default::default()
        }
    }

    pub fn pdf(filename: String, data: &[u8]) -> Self {
        Self {
            r#type: MultiPartMessageType::File,
            file: Some(InputFile {
                filename,
                file_data: format!("data:application/pdf;base64,{}", STANDARD.encode(data)),
            }),
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
pub struct StreamCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub error: Option<ErrorInfo>,
    pub model: Option<String>,
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
    #[serde(default)]
    pub cost: f64,
    pub cost_details: Option<DetailCost>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailCost {
    pub upstream_inference_cost: Option<f64>,
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
    Error,
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
    /// reasoning or summary of reasoning that should be display to user
    pub reasoning: Option<String>,
    /// alternative reasoning or summary of reasoning that should be display to user
    pub reasoning_content: Option<String>,
    /// reasoning or encrypted reasoning detail, useful for preserving-reasoning-blocks
    /// if multiple reasoning_details presented, concat them
    ///
    /// Reasoning are model-specific, meaning that when changing model, don't send them
    ///
    /// https://openrouter.ai/docs/guides/best-practices/reasoning-tokens#preserving-reasoning-blocks
    pub reasoning_details: Option<Vec<serde_json::Value>>,
    pub annotations: Option<Vec<serde_json::Value>>,
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub images: Vec<Image>,
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
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputMessage {
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub r#type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    // data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...
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
