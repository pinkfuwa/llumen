use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;

use super::configuration::{Configuration, ProcessState};
use crate::chat::prompt;
use crate::openrouter::{self, ReasoningEffort, Tool, ToolCall};

type ToolHandlerFn = dyn for<'a> Fn(&'a mut ProcessState, Vec<ToolCall>) -> BoxFuture<'a, Result<bool, anyhow::Error>>
    + Send
    + Sync;

/// Builder for Configuration with declarative API
pub struct ConfigurationBuilder {
    prompt: prompt::PromptKind,
    tools: Vec<Tool>,
    tool_filters: Vec<Box<dyn Fn(Vec<Tool>) -> Vec<Tool> + Send + Sync>>,
    tool_handler: Option<Arc<ToolHandlerFn>>,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
    reasoning_effort: Option<ReasoningEffort>,
    reasoning_max_tokens: Option<i32>,
    insert_web_search_context: bool,
    image_generation: bool,
}

impl ConfigurationBuilder {
    pub fn new(prompt: prompt::PromptKind) -> Self {
        Self {
            prompt,
            tools: Vec::new(),
            tool_filters: Vec::new(),
            tool_handler: None,
            max_tokens: None,
            temperature: None,
            reasoning_effort: None,
            reasoning_max_tokens: None,
            insert_web_search_context: false,
            image_generation: false,
        }
    }

    /// Add a tool to the configuration
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add multiple tools at once
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Apply a custom filter to tools if condition is true
    pub fn filter_tools_if<F>(mut self, condition: bool, filter: F) -> Self
    where
        F: Fn(Vec<Tool>) -> Vec<Tool> + Send + Sync + 'static,
    {
        if condition {
            self.tool_filters.push(Box::new(filter));
        }
        self
    }

    /// Remove a specific tool by name if condition is true
    pub fn remove_tool_if(self, condition: bool, tool_name: &'static str) -> Self {
        self.filter_tools_if(condition, move |tools| {
            tools.into_iter().filter(|t| t.name != tool_name).collect()
        })
    }

    /// Set the tool handler
    pub fn with_handler(mut self, handler: Arc<ToolHandlerFn>) -> Self {
        self.tool_handler = Some(handler);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set reasoning effort
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Set reasoning max tokens
    pub fn with_reasoning_max_tokens(mut self, max_tokens: i32) -> Self {
        self.reasoning_max_tokens = Some(max_tokens);
        self
    }

    /// Enable web search context insertion
    pub fn with_web_search_context(mut self, enabled: bool) -> Self {
        self.insert_web_search_context = enabled;
        self
    }

    /// Enable image generation
    pub fn with_image_generation(mut self, enabled: bool) -> Self {
        self.image_generation = enabled;
        self
    }

    /// Build the final Configuration
    pub fn build(self) -> Configuration {
        // Apply all filters to tools
        let mut tools = self.tools;
        for filter in self.tool_filters {
            tools = filter(tools);
        }

        // Default no-op handler if none provided
        let tool_handler = self
            .tool_handler
            .unwrap_or_else(|| Arc::new(|_, _| Box::pin(async { Ok(false) })));

        Configuration {
            completion_option: openrouter::CompletionOption {
                tools,
                max_tokens: self.max_tokens,
                temperature: self.temperature,
                reasoning_effort: self.reasoning_effort.unwrap_or_default(),
                reasoning_max_tokens: self.reasoning_max_tokens,
                insert_web_search_context: self.insert_web_search_context,
                image_generation: self.image_generation,
            },
            tool_handler,
            prompt: self.prompt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Normal).build();
        assert_eq!(config.completion_option.tools.len(), 0);
    }

    #[test]
    fn test_builder_with_tool() {
        let tool = Tool {
            name: "test_tool".into(),
            description: "test".into(),
            schema: Default::default(),
        };
        let config = ConfigurationBuilder::new(prompt::PromptKind::Normal)
            .with_tool(tool)
            .build();
        assert_eq!(config.completion_option.tools.len(), 1);
        assert_eq!(config.completion_option.tools[0].name, "test_tool");
    }

    #[test]
    fn test_builder_with_multiple_tools() {
        let tools = vec![
            Tool {
                name: "tool1".into(),
                description: "test".into(),
                schema: Default::default(),
            },
            Tool {
                name: "tool2".into(),
                description: "test".into(),
                schema: Default::default(),
            },
        ];
        let config = ConfigurationBuilder::new(prompt::PromptKind::Normal)
            .with_tools(tools)
            .build();
        assert_eq!(config.completion_option.tools.len(), 2);
    }

    #[test]
    fn test_builder_remove_tool_if_true() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Search)
            .with_tool(Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .with_tool(Tool {
                name: "crawl".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .remove_tool_if(true, "web_search")
            .build();

        assert_eq!(config.completion_option.tools.len(), 1);
        assert_eq!(config.completion_option.tools[0].name, "crawl");
    }

    #[test]
    fn test_builder_remove_tool_if_false() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Search)
            .with_tool(Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .with_tool(Tool {
                name: "crawl".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .remove_tool_if(false, "web_search")
            .build();

        assert_eq!(config.completion_option.tools.len(), 2);
    }

    #[test]
    fn test_builder_multiple_filters() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Search)
            .with_tool(Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .with_tool(Tool {
                name: "crawl".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .with_tool(Tool {
                name: "other".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .remove_tool_if(true, "web_search")
            .remove_tool_if(true, "crawl")
            .build();

        assert_eq!(config.completion_option.tools.len(), 1);
        assert_eq!(config.completion_option.tools[0].name, "other");
    }

    #[test]
    fn test_builder_custom_filter() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Normal)
            .with_tool(Tool {
                name: "tool1".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .with_tool(Tool {
                name: "tool2".into(),
                description: "test".into(),
                schema: Default::default(),
            })
            .filter_tools_if(true, |tools| {
                tools
                    .into_iter()
                    .filter(|t| t.name.starts_with("tool1"))
                    .collect()
            })
            .build();

        assert_eq!(config.completion_option.tools.len(), 1);
        assert_eq!(config.completion_option.tools[0].name, "tool1");
    }

    #[test]
    fn test_builder_with_options() {
        let config = ConfigurationBuilder::new(prompt::PromptKind::Normal)
            .with_max_tokens(1000)
            .with_temperature(0.7)
            .with_reasoning_max_tokens(500)
            .with_web_search_context(true)
            .build();

        assert_eq!(config.completion_option.max_tokens, Some(1000));
        assert_eq!(config.completion_option.temperature, Some(0.7));
        assert_eq!(config.completion_option.reasoning_max_tokens, Some(500));
        assert_eq!(config.completion_option.insert_web_search_context, true);
    }
}
