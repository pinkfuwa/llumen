use super::raw;

#[derive(Clone, Copy, Default)]
pub enum ReasoningEffort {
    None,
    Low,
    Medium,
    High,
    #[default]
    Auto,
}

impl ReasoningEffort {
    pub fn to_value(&self) -> Option<String> {
        match self {
            ReasoningEffort::None => Some("none".to_string()),
            ReasoningEffort::Low => Some("low".to_string()),
            ReasoningEffort::Medium => Some("medium".to_string()),
            ReasoningEffort::High => Some("high".to_string()),
            ReasoningEffort::Auto => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct CompletionOption {
    pub insert_web_search_context: bool,
    pub image_generation: bool,
    pub max_tokens: Option<i32>,
    pub reasoning_effort: ReasoningEffort,
    pub tools: Vec<Tool>,
}

impl CompletionOption {
    pub fn builder() -> OptionBuilder {
        OptionBuilder::default()
    }
    pub fn tools(tools: &[Tool]) -> Self {
        let mut self_ = Self::default();
        for tool in tools {
            self_.tools.push(tool.clone());
        }
        self_
    }
}

#[derive(Default)]
pub struct OptionBuilder {
    insert_web_search_context: bool,
    image_generation: bool,
    max_tokens: Option<i32>,
    reasoning_effort: ReasoningEffort,
    tools: Vec<Tool>,
}

impl OptionBuilder {
    pub fn web_search(mut self, enable: bool) -> Self {
        self.insert_web_search_context = enable;
        self
    }

    pub fn image_generation(mut self, enable: bool) -> Self {
        self.image_generation = enable;
        self
    }

    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = effort;
        self
    }

    pub fn tools(mut self, tools: &[Tool]) -> Self {
        for tool in tools {
            self.tools.push(tool.clone());
        }
        self
    }

    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn build(self) -> CompletionOption {
        CompletionOption {
            insert_web_search_context: self.insert_web_search_context,
            image_generation: self.image_generation,
            max_tokens: self.max_tokens,
            reasoning_effort: self.reasoning_effort,
            tools: self.tools,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

impl From<Tool> for raw::Tool {
    fn from(tool: Tool) -> Self {
        raw::Tool {
            r#type: "function".to_string(),
            function: raw::FunctionTool {
                name: tool.name,
                description: tool.description,
                parameters: tool.schema,
            },
        }
    }
}
