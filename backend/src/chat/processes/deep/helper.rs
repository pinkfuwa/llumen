use serde::{Deserialize, Serialize};

/// Planner response structure matching the prompt output
#[derive(Debug, Clone, Deserialize, Serialize)]
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
