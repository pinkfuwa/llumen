use entity::{ChunkKind, chunk, patch};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};

use crate::openrouter;

/// Planner response structure matching the prompt output
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PlannerResponse {
    pub locale: String,
    pub has_enough_context: bool,
    pub thought: String,
    pub title: String,
    pub steps: Vec<PlannerStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlannerStep {
    pub need_search: bool,
    pub title: String,
    pub description: String,
    pub step_type: String,
}

/// Handoff tool call structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HandoffToPlanner {
    pub research_topic: String,
    pub local: String,
}

pub fn tool_chunk(content: &patch::ToolCall) -> chunk::ActiveModel {
    chunk::ActiveModel {
        content: ActiveValue::Set(serde_json::to_string(&content).unwrap()),
        kind: ActiveValue::Set(ChunkKind::ToolCall),
        ..Default::default()
    }
}

pub fn get_web_search_tool_def() -> openrouter::Tool {
    todo!()
}

pub fn get_crawl_tool_def() -> openrouter::Tool {
    todo!()
}

pub fn get_lua_repl_def() -> openrouter::Tool {
    todo!()
}
