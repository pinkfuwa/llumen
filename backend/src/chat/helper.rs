//! Helper types for deep-research structured outputs.

use protocol::{Deep, Step, StepKind};
use schemars::JsonSchema;
use serde::Deserialize;

/// Response shape from the planner LLM (structured output).
#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
pub struct PlannerResponse {
    pub locale: String,
    pub has_enough_context: bool,
    pub thought: String,
    pub title: String,
    #[serde(default)]
    pub steps: Vec<PlannerStep>,
}

#[derive(Debug, Deserialize, serde::Serialize, JsonSchema)]
pub struct PlannerStep {
    pub need_search: bool,
    pub title: String,
    pub description: String,
    #[serde(default = "default_step_type")]
    pub step_type: String,
}

fn default_step_type() -> String {
    "research".to_string()
}

impl From<PlannerResponse> for Deep {
    fn from(resp: PlannerResponse) -> Self {
        Deep {
            locale: resp.locale,
            has_enough_context: resp.has_enough_context,
            thought: resp.thought,
            title: resp.title,
            steps: resp
                .steps
                .into_iter()
                .map(|s| Step {
                    need_search: s.need_search,
                    title: s.title,
                    description: s.description,
                    kind: if s.step_type == "processing" {
                        StepKind::Code
                    } else {
                        StepKind::Research
                    },
                    progress: Vec::new(),
                })
                .collect(),
        }
    }
}
