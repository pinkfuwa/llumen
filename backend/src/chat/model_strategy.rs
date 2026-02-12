use std::sync::Arc;

use crate::openrouter::{Capability, Message, Tool};

/// Strategy pattern for model-specific behavior
pub trait ModelStrategy: Send + Sync {
    /// Should context message be injected for this model?
    fn should_inject_context(&self) -> bool;
    
    /// Filter tools based on model capabilities
    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool>;
    
    /// Transform messages before sending to API (for future use)
    fn prepare_messages(&self, messages: Vec<Message>) -> Vec<Message> {
        messages
    }
}

/// Strategy for text-only models (e.g., GPT-4, Claude)
pub struct TextOnlyStrategy;

impl ModelStrategy for TextOnlyStrategy {
    fn should_inject_context(&self) -> bool {
        true
    }
    
    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool> {
        tools
    }
}

/// Strategy for image-output-only models (e.g., FLUX2)
pub struct ImageOnlyStrategy;

impl ModelStrategy for ImageOnlyStrategy {
    fn should_inject_context(&self) -> bool {
        false // Image-only models should NOT receive context message
    }
    
    fn filter_tools(&self, _tools: Vec<Tool>) -> Vec<Tool> {
        // Image generation models typically don't use tools
        vec![]
    }
}

/// Strategy for mixed text/image models (e.g., gemini-image-gen-3-pro)
pub struct MixedStrategy;

impl ModelStrategy for MixedStrategy {
    fn should_inject_context(&self) -> bool {
        true
    }
    
    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool> {
        tools
    }
}

/// Factory function to get appropriate strategy based on model capabilities
pub fn get_model_strategy(capability: &Capability) -> Arc<dyn ModelStrategy> {
    match (capability.text_output, capability.image_output) {
        (false, true) => Arc::new(ImageOnlyStrategy),
        (true, false) => Arc::new(TextOnlyStrategy),
        (true, true) => Arc::new(MixedStrategy),
        (false, false) => Arc::new(TextOnlyStrategy), // fallback to text-only
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::OcrEngine;

    #[test]
    fn test_image_only_strategy_no_context() {
        let strategy = ImageOnlyStrategy;
        assert_eq!(strategy.should_inject_context(), false);
    }

    #[test]
    fn test_image_only_strategy_no_tools() {
        let strategy = ImageOnlyStrategy;
        let tools = vec![
            Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            },
        ];
        let filtered = strategy.filter_tools(tools);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_text_only_strategy_with_context() {
        let strategy = TextOnlyStrategy;
        assert_eq!(strategy.should_inject_context(), true);
    }

    #[test]
    fn test_text_only_strategy_keeps_tools() {
        let strategy = TextOnlyStrategy;
        let tools = vec![
            Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            },
        ];
        let filtered = strategy.filter_tools(tools.clone());
        assert_eq!(filtered.len(), tools.len());
    }

    #[test]
    fn test_mixed_strategy_with_context() {
        let strategy = MixedStrategy;
        assert_eq!(strategy.should_inject_context(), true);
    }

    #[test]
    fn test_mixed_strategy_keeps_tools() {
        let strategy = MixedStrategy;
        let tools = vec![
            Tool {
                name: "web_search".into(),
                description: "test".into(),
                schema: Default::default(),
            },
        ];
        let filtered = strategy.filter_tools(tools.clone());
        assert_eq!(filtered.len(), tools.len());
    }

    #[test]
    fn test_get_model_strategy_image_only() {
        let capability = Capability {
            text_output: false,
            image_output: true,
            image_input: false,
            structured_output: false,
            toolcall: false,
            ocr: OcrEngine::Disabled,
            audio: false,
            reasoning: false,
        };
        let strategy = get_model_strategy(&capability);
        assert_eq!(strategy.should_inject_context(), false);
    }

    #[test]
    fn test_get_model_strategy_text_only() {
        let capability = Capability {
            text_output: true,
            image_output: false,
            image_input: false,
            structured_output: false,
            toolcall: false,
            ocr: OcrEngine::Disabled,
            audio: false,
            reasoning: false,
        };
        let strategy = get_model_strategy(&capability);
        assert_eq!(strategy.should_inject_context(), true);
    }

    #[test]
    fn test_get_model_strategy_mixed() {
        let capability = Capability {
            text_output: true,
            image_output: true,
            image_input: false,
            structured_output: false,
            toolcall: false,
            ocr: OcrEngine::Disabled,
            audio: false,
            reasoning: false,
        };
        let strategy = get_model_strategy(&capability);
        assert_eq!(strategy.should_inject_context(), true);
    }

    #[test]
    fn test_get_model_strategy_fallback_no_outputs() {
        let capability = Capability {
            text_output: false,
            image_output: false,
            image_input: false,
            structured_output: false,
            toolcall: false,
            ocr: OcrEngine::Disabled,
            audio: false,
            reasoning: false,
        };
        let strategy = get_model_strategy(&capability);
        assert_eq!(strategy.should_inject_context(), true); // fallback to text-only
    }
}
