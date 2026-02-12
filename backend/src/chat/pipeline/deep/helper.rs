use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Planner response structure matching the prompt output
#[derive(Debug, Clone, Deserialize, Serialize, Default, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PlannerResponse {
    pub locale: String,
    pub has_enough_context: bool,
    pub thought: String,
    pub title: String,
    pub steps: Vec<PlannerStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct PlannerStep {
    pub need_search: bool,
    pub title: String,
    pub description: String,
    pub step_type: String,
}

impl From<PlannerResponse> for protocol::Deep {
    fn from(value: PlannerResponse) -> Self {
        protocol::Deep {
            locale: value.locale,
            has_enough_context: value.has_enough_context,
            thought: value.thought,
            title: value.title,
            steps: value
                .steps
                .into_iter()
                .map(|step| protocol::Step {
                    need_search: step.need_search,
                    title: step.title,
                    description: step.description,
                    kind: match step.step_type.to_lowercase().as_str() {
                        "code" => protocol::StepKind::Code,
                        "research" => protocol::StepKind::Research,
                        _ => protocol::StepKind::Research, // Default to Research
                    },
                    progress: Vec::new(),
                })
                .collect(),
        }
    }
}

// TODO: provide different prompt to different topic
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct HandoffToPlanner {
//     pub research_topic: String,
//     pub local: String,
// }
