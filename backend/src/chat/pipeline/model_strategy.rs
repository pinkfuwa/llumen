use std::sync::Arc;

use crate::openrouter::{Capability, Tool};

/// Strategy pattern for model-specific behavior.
///
/// Different model types need different treatment:
/// - Text-only models (GPT-4, Claude): get context, keep tools
/// - Image-only models (FLUX2): skip context, skip tools
/// - Mixed models (gemini-image-gen-3-pro): get context, keep tools
pub trait ModelStrategy: Send + Sync {
    fn should_inject_context(&self) -> bool;
    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool>;
}

pub struct TextOnlyStrategy;

impl ModelStrategy for TextOnlyStrategy {
    fn should_inject_context(&self) -> bool {
        true
    }

    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool> {
        tools
    }
}

pub struct ImageOnlyStrategy;

impl ModelStrategy for ImageOnlyStrategy {
    fn should_inject_context(&self) -> bool {
        false
    }

    fn filter_tools(&self, _tools: Vec<Tool>) -> Vec<Tool> {
        vec![]
    }
}

pub struct MixedStrategy;

impl ModelStrategy for MixedStrategy {
    fn should_inject_context(&self) -> bool {
        true
    }

    fn filter_tools(&self, tools: Vec<Tool>) -> Vec<Tool> {
        tools
    }
}

pub fn get_model_strategy(capability: &Capability) -> Arc<dyn ModelStrategy> {
    match (capability.text_output, capability.image_output) {
        (false, true) => Arc::new(ImageOnlyStrategy),
        (true, true) => Arc::new(MixedStrategy),
        _ => Arc::new(TextOnlyStrategy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::OcrEngine;

    fn make_capability(text: bool, image: bool) -> Capability {
        Capability {
            text_output: text,
            image_output: image,
            image_input: false,
            structured_output: false,
            toolcall: false,
            ocr: OcrEngine::Disabled,
            audio: false,
            reasoning: false,
        }
    }

    #[test]
    fn image_only_model_skips_context_and_tools() {
        let strategy = get_model_strategy(&make_capability(false, true));
        assert!(!strategy.should_inject_context());
        assert!(
            strategy
                .filter_tools(vec![Tool {
                    name: "test".into(),
                    description: "test".into(),
                    schema: Default::default(),
                }])
                .is_empty()
        );
    }

    #[test]
    fn text_only_model_gets_context_and_tools() {
        let strategy = get_model_strategy(&make_capability(true, false));
        assert!(strategy.should_inject_context());
        assert_eq!(
            strategy
                .filter_tools(vec![Tool {
                    name: "test".into(),
                    description: "test".into(),
                    schema: Default::default(),
                }])
                .len(),
            1
        );
    }

    #[test]
    fn mixed_model_gets_context_and_tools() {
        let strategy = get_model_strategy(&make_capability(true, true));
        assert!(strategy.should_inject_context());
        assert_eq!(
            strategy
                .filter_tools(vec![Tool {
                    name: "test".into(),
                    description: "test".into(),
                    schema: Default::default(),
                }])
                .len(),
            1
        );
    }
}
