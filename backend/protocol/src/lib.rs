use sea_orm::{DeriveActiveEnum, EnumIter, FromJsonQueryResult};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Code,
    Research,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[typeshare]
pub struct Deep {
    pub locale: String,
    pub has_enough_context: bool,
    pub thought: String,
    pub title: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[typeshare]
pub struct Step {
    pub need_search: bool,
    pub title: String,
    pub description: String,
    pub kind: StepKind,
    pub progress: Vec<AssistantChunk>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum AssistantChunk {
    Annotation(String),
    Text(String),
    Reasoning(String),
    ToolCall {
        id: String,
        arg: String,
        name: String,
    },
    ToolResult {
        id: String,
        response: String,
    },
    Error(String),
    DeepAgent(Deep),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[typeshare]
pub struct FileMetadata {
    pub name: String,
    pub id: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, FromJsonQueryResult)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum MessageInner {
    User {
        text: String,
        files: Vec<FileMetadata>,
    },
    Assistant(Vec<AssistantChunk>),
}

impl Default for MessageInner {
    fn default() -> Self {
        MessageInner::Assistant(Vec::new())
    }
}

impl AssistantChunk {
    pub fn as_deep(&mut self) -> Option<&mut Deep> {
        if let AssistantChunk::DeepAgent(deep) = self {
            Some(deep)
        } else {
            None
        }
    }
    pub fn as_text(&self) -> Option<&String> {
        if let AssistantChunk::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }
    pub fn as_annotation(&self) -> Option<&String> {
        if let AssistantChunk::Annotation(annotation) = self {
            Some(annotation)
        } else {
            None
        }
    }
}

impl MessageInner {
    pub fn as_assistant(&mut self) -> Option<&mut Vec<AssistantChunk>> {
        if let MessageInner::Assistant(assistant_chunks) = self {
            Some(assistant_chunks)
        } else {
            None
        }
    }
    pub fn last_assistant(&mut self) -> Option<&mut AssistantChunk> {
        if let MessageInner::Assistant(assistant_chunks) = self {
            assistant_chunks.last_mut()
        } else {
            None
        }
    }
    pub fn add_error(&mut self, msg: String) {
        match self {
            MessageInner::User { .. } => {}
            MessageInner::Assistant(assistant_chunks) => {
                assistant_chunks.push(AssistantChunk::Error(msg))
            }
        };
    }
    pub fn add_annotation(&mut self, json_str: String) {
        match self {
            MessageInner::User { .. } => {}
            MessageInner::Assistant(assistant_chunks) => {
                assistant_chunks.push(AssistantChunk::Annotation(json_str))
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            MessageInner::User { .. } => false,
            MessageInner::Assistant(chunks) => chunks.is_empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[typeshare]
pub struct UserPreference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_on_enter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default, Serialize)]
pub enum OcrEngine {
    Native,
    Text,
    Mistral,
    #[default]
    Disabled,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, PartialEq)]
pub struct ModelCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr: Option<OcrEngine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, PartialEq)]
pub struct ModelParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModelConfig {
    pub display_name: String,
    pub model_id: String,
    #[serde(default)]
    pub capability: ModelCapability,
    #[serde(default)]
    pub parameter: ModelParameter,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ModeKind {
    Normal = 0,
    Search = 1,
    Research = 3,
}
