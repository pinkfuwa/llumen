//! High-level options without mapping.
//!
//! No-mapping is used here.

use super::raw;

/// How much reasoning effort the model should use before generating a response.
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
    /// Returns the API string value, or `None` for `Auto`.
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

/// Options for a completion request to OpenRouter.
#[derive(Clone, Default)]
pub struct CompletionOption {
    pub web_plugin_search: bool,
    pub image_generation: bool,
    pub max_tokens: Option<i32>,
    pub reasoning_effort: ReasoningEffort,
    pub tools: Vec<Tool>,
    pub temperature: Option<f32>,
    pub reasoning_max_tokens: Option<i32>,
    pub session_id: Option<String>,
}

impl CompletionOption {
    /// Creates a new [`OptionBuilder`] with default values.
    pub fn builder() -> OptionBuilder {
        OptionBuilder::default()
    }
    /// Constructs a `CompletionOption` with the given tools and defaults for
    /// all other fields.
    pub fn tools(tools: &[Tool]) -> Self {
        let mut self_ = Self::default();
        for tool in tools {
            self_.tools.push(tool.clone());
        }
        self_
    }
}

/// Builder for constructing a [`CompletionOption`] step by step.
#[derive(Default)]
pub struct OptionBuilder {
    web_plugin_search: bool,
    image_generation: bool,
    max_tokens: Option<i32>,
    reasoning_effort: ReasoningEffort,
    tools: Vec<Tool>,
    temperature: Option<f32>,
    reasoning_max_tokens: Option<i32>,
    session_id: Option<String>,
}

impl OptionBuilder {
    /// Enables or disables web search plugin.
    pub fn web_search(mut self, enable: bool) -> Self {
        self.web_plugin_search = enable;
        self
    }

    /// Enables or disables image generation.
    pub fn image_generation(mut self, enable: bool) -> Self {
        self.image_generation = enable;
        self
    }

    /// Sets the maximum number of tokens the model can generate.
    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the maximum number of tokens the model can use for reasoning.
    pub fn max_reasoning_tokens(mut self, max_tokens: i32) -> Self {
        self.reasoning_max_tokens = Some(max_tokens);
        self
    }

    /// Sets the reasoning effort level for the model.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = effort;
        self
    }

    /// Adds all tools from the given slice to the request.
    pub fn tools(mut self, tools: &[Tool]) -> Self {
        for tool in tools {
            self.tools.push(tool.clone());
        }
        self
    }

    /// Adds a single tool to the request.
    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Sets the sampling temperature for the model.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Associates a session ID for conversation continuity.
    pub fn session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Consumes the builder and produces a [`CompletionOption`].
    pub fn build(self) -> CompletionOption {
        CompletionOption {
            web_plugin_search: self.web_plugin_search,
            image_generation: self.image_generation,
            max_tokens: self.max_tokens,
            reasoning_effort: self.reasoning_effort,
            tools: self.tools,
            temperature: self.temperature,
            reasoning_max_tokens: self.reasoning_max_tokens,
            session_id: self.session_id,
        }
    }
}

/// A function tool definition that the model may call.
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
            function: Some(raw::FunctionTool {
                name: tool.name,
                description: tool.description,
                parameters: tool.schema,
            }),
        }
    }
}
