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
    openrouter::Tool {
        name: "web_search_tool".to_string(),
        description: "Search the web for information using a search query.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to use for finding information on the web."
                }
            },
            "required": ["query"]
        }),
    }
}

pub fn get_crawl_tool_def() -> openrouter::Tool {
    openrouter::Tool {
        name: "crawl_tool".to_string(),
        description: "Crawl and extract content from a specific URL.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to crawl and extract content from."
                }
            },
            "required": ["url"]
        }),
    }
}

pub fn get_lua_repl_def() -> openrouter::Tool {
    openrouter::Tool {
        name: "lua_repl".to_string(),
        description: "Execute lua code and do data analysis or calculation. If you want to see the output of a value, you should print it out with `print(...)`. This is visible to the user.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The lua code to execute to do further analysis or calculation."
                }
            },
            "required": ["code"]
        }),
    }
}
