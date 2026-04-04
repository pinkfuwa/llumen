//! OpenRouter API request and response structures
//!
//! Not codegen, but it match the API spec
//!
//! https://openrouter.ai/docs
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use stream_json::{Base64EmbedFile, IntoSerializer, Serializer};

use crate::utils::blob::BlobReader;

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

#[derive(Debug, Clone, IntoSerializer)]
pub struct CompletionReq {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[stream(skip_serialize_if = "|value: &Option<f32>| value.is_none()")]
    pub temperature: Option<f32>,
    #[stream(skip_serialize_if = "|value: &Option<f32>| value.is_none()")]
    pub repeat_penalty: Option<f32>,
    #[stream(skip_serialize_if = "|value: &Option<i32>| value.is_none()")]
    pub top_k: Option<i32>,
    #[stream(skip_serialize_if = "|value: &Option<f32>| value.is_none()")]
    pub top_p: Option<f32>,
    #[stream(skip_serialize_if = "|value: &Option<i32>| value.is_none()")]
    pub max_tokens: Option<i32>,
    #[stream(skip_serialize_if = "|value: &Vec<Tool>| value.is_empty()")]
    pub tools: Vec<Tool>,
    #[stream(skip_serialize_if = "|value: &Vec<Plugin>| value.is_empty()")]
    pub plugins: Vec<Plugin>,
    #[stream(skip_serialize_if = "|value: &Option<WebSearchOptions>| value.is_none()")]
    pub web_search_options: Option<WebSearchOptions>,
    #[stream(skip_serialize_if = "|value: &Option<UsageReq>| value.is_none()")]
    pub usage: Option<UsageReq>,
    #[stream(skip_serialize_if = "|value: &Option<ResponseFormat>| value.is_none()")]
    pub response_format: Option<ResponseFormat>,
    // reasoning options
    #[stream(skip_serialize_if = "|value: &Reasoning| value.is_empty()")]
    pub reasoning: Reasoning,
    #[stream(skip_serialize_if = "|value: &Vec<String>| value.is_empty()")]
    pub modalities: Vec<String>,
}

#[derive(Debug, Clone, IntoSerializer, Default)]
pub struct Reasoning {
    #[stream(skip_serialize_if = "|value: &Option<String>| value.is_none()")]
    pub effort: Option<String>,
    #[stream(skip_serialize_if = "|value: &Option<bool>| value.is_none()")]
    pub enabled: Option<bool>,
    #[stream(skip_serialize_if = "|value: &Option<i32>| value.is_none()")]
    pub max_tokens: Option<i32>,
}

impl Reasoning {
    pub fn is_empty(&self) -> bool {
        self.effort.is_none() && self.enabled.is_none()
    }

    /// Set reasoning field to compatible mode
    ///
    /// Only effort is part of baseline -> we assume every provide support
    /// `effort`.
    pub fn set_compatible(&mut self) {
        self.enabled = None;
        self.max_tokens = None;
    }
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct ResponseFormat {
    pub r#type: String,
    pub json_schema: String,
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct UsageReq {
    pub include: bool,
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct Plugin {
    pub id: String,
    #[stream(skip_serialize_if = "|value: &Option<PdfPlugin>| value.is_none()")]
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

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct WebSearchOptions {
    #[stream(rename = "search_context_size")]
    pub search_context_size: String,
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct PdfPlugin {
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct Tool {
    pub r#type: String,
    pub function: FunctionTool,
}

#[derive(Debug, Clone, Serialize, stream_json::IntoSerializer)]
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// TODO: have both content and contents set will cause serialization error
#[derive(Debug, Clone, Default, stream_json::IntoSerializer)]
pub struct Message {
    pub role: Role,
    #[stream(skip_serialize_if = "|value: &Option<String>| value.is_none()")]
    pub content: Option<String>,
    #[stream(skip_serialize_if = "|value: &Option<Vec<ToolCallReq>>| value.is_none()")]
    pub tool_calls: Option<Vec<ToolCallReq>>,
    #[stream(skip_serialize_if = "|value: &Option<String>| value.is_none()")]
    pub tool_call_id: Option<String>,
    #[stream(
        rename = "content",
        skip_serialize_if = "|value: &Option<Vec<MessagePart>>| value.is_none()"
    )]
    pub contents: Option<Vec<MessagePart>>,
    #[stream(skip_serialize_if = "|value: &Option<serde_json::Value>| value.is_none()")]
    pub annotations: Option<serde_json::Value>,
    // reasoning text or encrypted reasoning detail
    #[stream(skip_serialize_if = "|value: &Vec<serde_json::Value>| value.is_empty()")]
    pub reasoning_details: Vec<serde_json::Value>,
}

// `data:image/jpeg;base64,${base64Image}`;
#[derive(Debug, Clone, Serialize, Default, stream_json::IntoSerializer)]
pub enum MultiPartMessageType {
    #[default]
    Text,
    ImageUrl,
    File,
    InputAudio,
}

#[derive(Debug, Clone, Default, stream_json::IntoSerializer)]
pub struct MessagePart {
    pub r#type: MultiPartMessageType,
    #[stream(skip_serialize_if = "|value: &Option<String>| value.is_none()")]
    pub text: Option<String>,
    #[stream(skip_serialize_if = "|value: &Option<InputAudio>| value.is_none()")]
    pub input_audio: Option<InputAudio>,
    #[stream(skip_serialize_if = "|value: &Option<InputFile>| value.is_none()")]
    pub file: Option<InputFile>,
    #[stream(skip_serialize_if = "|value: &Option<InputImage>| value.is_none()")]
    pub image_url: Option<InputImage>,
}

#[derive(Debug, Clone)]
pub struct FileDataUri {
    pub data: BlobReader,
    pub size: usize,
    pub mime_type: String,
}

impl FileDataUri {
    pub fn new(data: BlobReader, mime_type: String) -> Self {
        let size = data.len();
        Self {
            data,
            size,
            mime_type,
        }
    }

    pub fn audio_format(&self) -> String {
        self.mime_type
            .rsplit('/')
            .next()
            .unwrap_or("wav")
            .to_string()
    }
}

impl IntoSerializer for FileDataUri {
    type S = Base64EmbedFile<BlobReader>;

    fn into_serializer(self) -> Self::S {
        match Base64EmbedFile::new(self.data, self.size, self.mime_type) {
            Ok(serializer) => serializer,
            Err(_) => unreachable!("Base64EmbedFile::new is infallible"),
        }
    }

    fn size(&self) -> Option<usize> {
        Base64EmbedFile::new(self.data.clone(), self.size, self.mime_type.clone())
            .ok()
            .and_then(|serializer| serializer.size())
    }
}

#[derive(Debug, Clone)]
pub enum MaybeEmbedded {
    Plain(String),
    Embedded(FileDataUri),
}

pub enum MaybeEmbeddedSerializer {
    Plain(<String as IntoSerializer>::S),
    Embedded(<FileDataUri as IntoSerializer>::S),
}

impl Serializer for MaybeEmbeddedSerializer {
    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Bytes, stream_json::Error>>> {
        match self {
            MaybeEmbeddedSerializer::Plain(serializer) => serializer.poll(cx),
            MaybeEmbeddedSerializer::Embedded(serializer) => serializer.poll(cx),
        }
    }
}

impl Unpin for MaybeEmbeddedSerializer {}

impl IntoSerializer for MaybeEmbedded {
    type S = MaybeEmbeddedSerializer;

    fn into_serializer(self) -> Self::S {
        match self {
            MaybeEmbedded::Plain(value) => MaybeEmbeddedSerializer::Plain(value.into_serializer()),
            MaybeEmbedded::Embedded(value) => {
                MaybeEmbeddedSerializer::Embedded(value.into_serializer())
            }
        }
    }

    fn size(&self) -> Option<usize> {
        match self {
            MaybeEmbedded::Plain(value) => value.size(),
            MaybeEmbedded::Embedded(value) => value.size(),
        }
    }
}

impl MessagePart {
    pub fn text(text: String) -> Self {
        Self {
            r#type: MultiPartMessageType::Text,
            text: Some(text),
            ..Default::default()
        }
    }

    pub fn from_file(file: super::message::File) -> (Self, Self) {
        let super::message::File {
            name,
            mime_type,
            data,
        } = file;

        if mime_type == "application/pdf" {
            return (
                Self::text(format!("Uploaded file: {}", name)),
                Self::pdf(name, data),
            );
        }

        if mime_type.starts_with("image/") {
            return (
                Self::text(format!("Uploaded file: {}", name)),
                Self::image_data(data, mime_type),
            );
        }

        if mime_type.starts_with("audio/") {
            return (
                Self::text(format!("Uploaded file: {}", name)),
                Self::input_audio(data, mime_type),
            );
        }

        (
            Self::text(format!("Uploaded file: {}", name)),
            Self::file(name, data, mime_type),
        )
    }

    pub fn image_url(url: String) -> Self {
        Self {
            r#type: MultiPartMessageType::ImageUrl,
            image_url: Some(InputImage {
                url: MaybeEmbedded::Plain(url),
            }),
            ..Default::default()
        }
    }

    pub fn image_data(data: BlobReader, mime_type: String) -> Self {
        Self {
            r#type: MultiPartMessageType::ImageUrl,
            image_url: Some(InputImage {
                url: MaybeEmbedded::Embedded(FileDataUri::new(data, mime_type)),
            }),
            ..Default::default()
        }
    }

    pub fn pdf(filename: String, data: BlobReader) -> Self {
        Self {
            r#type: MultiPartMessageType::File,
            file: Some(InputFile {
                filename,
                file_data: FileDataUri::new(data, "application/pdf".to_string()),
            }),
            ..Default::default()
        }
    }

    pub fn file(filename: String, file_data: BlobReader, mime_type: String) -> Self {
        Self {
            r#type: MultiPartMessageType::File,
            file: Some(InputFile {
                filename,
                file_data: FileDataUri::new(file_data, mime_type),
            }),
            ..Default::default()
        }
    }

    pub fn input_audio(data: BlobReader, mime_type: String) -> Self {
        let data = FileDataUri::new(data, mime_type);
        Self {
            r#type: MultiPartMessageType::InputAudio,
            input_audio: Some(InputAudio {
                format: data.audio_format(),
                data,
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, stream_json::IntoSerializer)]
pub struct InputAudio {
    pub data: FileDataUri,
    pub format: String,
}

#[derive(Debug, Clone, stream_json::IntoSerializer)]
pub struct InputFile {
    pub filename: String,
    pub file_data: FileDataUri,
}

#[derive(Debug, Clone, stream_json::IntoSerializer)]
pub struct InputImage {
    pub url: MaybeEmbedded,
}

#[derive(Debug, Clone, stream_json::IntoSerializer)]
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

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, stream_json::IntoSerializer,
)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, stream_json::IntoSerializer)]
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
    /// alternative reasoning or summary of reasoning that should be display to
    /// user
    pub reasoning_content: Option<String>,
    /// reasoning or encrypted reasoning detail, useful for
    /// preserving-reasoning-blocks if multiple reasoning_details presented,
    /// concat them
    ///
    /// Reasoning are model-specific, meaning that when changing model, don't
    /// send them
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

#[derive(Debug, Clone, Deserialize, IntoSerializer)]
pub struct ToolFunctionResp {
    pub arguments: Option<String>,
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

#[derive(serde::Deserialize)]
pub struct EmbeddingModel {
    pub id: String,
    pub name: Option<String>,
}

// https://openrouter.ai/api/v1/embeddings/models
#[derive(serde::Deserialize)]
pub struct EmbeddingModelListResponse {
    pub data: Vec<EmbeddingModel>,
}

#[derive(Debug, Clone, IntoSerializer)]
pub struct EmbeddingReq {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, IntoSerializer)]
pub struct EmbeddingBatchReq {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
    pub index: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    pub price: f64,
    pub data: Vec<EmbeddingResult>,
    pub usage: Option<Usage>,
}
