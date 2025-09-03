use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CompletionReq {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
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

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallReq>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    Tool,
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
