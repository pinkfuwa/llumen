use crate::openrouter::{self, Message};

use super::model_strategy::{self, ModelStrategy};

/// Builds the message list for an LLM completion request.
///
/// This is a pure builder — no database, no network, no side effects.
/// That makes it easy to test and reason about.
///
/// # Example
/// ```ignore
/// let messages = MessageBuilder::new(system_prompt)
///     .history(converted_history)
///     .context(&strategy, context_prompt)
///     .build();
/// ```
pub struct MessageBuilder {
    messages: Vec<Message>,
}

impl MessageBuilder {
    /// Start with a system prompt.
    pub fn new(system_prompt: String) -> Self {
        Self {
            messages: vec![Message::System(system_prompt)],
        }
    }

    /// Append previously-converted chat history.
    pub fn history(mut self, msgs: impl IntoIterator<Item = Message>) -> Self {
        self.messages.extend(msgs);
        self
    }

    /// Append context prompt ONLY if the model strategy allows it.
    ///
    /// Image-only models (like FLUX2) return `should_inject_context() == false`,
    /// so context is automatically skipped for them. Mixed models (like
    /// gemini-image-gen-3-pro) still receive context because they can process text.
    pub fn context(mut self, strategy: &dyn ModelStrategy, context_prompt: String) -> Self {
        if strategy.should_inject_context() {
            self.messages.push(Message::User(context_prompt));
        }
        self
    }

    pub fn build(self) -> Vec<Message> {
        self.messages
    }
}

/// Convenience: resolve the right strategy from a capability and build messages.
pub fn build_messages(
    system_prompt: String,
    history: Vec<Message>,
    context_prompt: String,
    capability: &openrouter::Capability,
) -> Vec<Message> {
    let strategy = model_strategy::get_model_strategy(capability);
    MessageBuilder::new(system_prompt)
        .history(history)
        .context(strategy.as_ref(), context_prompt)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::pipeline::model_strategy::{ImageOnlyStrategy, MixedStrategy, TextOnlyStrategy};

    #[test]
    fn text_model_gets_context() {
        let msgs = MessageBuilder::new("system".into())
            .history(vec![Message::User("hello".into())])
            .context(&TextOnlyStrategy, "ctx".into())
            .build();

        // system + user + context = 3
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn image_only_model_skips_context() {
        let msgs = MessageBuilder::new("system".into())
            .history(vec![Message::User("hello".into())])
            .context(&ImageOnlyStrategy, "ctx".into())
            .build();

        // system + user = 2, NO context
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn mixed_model_gets_context() {
        let msgs = MessageBuilder::new("system".into())
            .history(vec![Message::User("hello".into())])
            .context(&MixedStrategy, "ctx".into())
            .build();

        // system + user + context = 3
        assert_eq!(msgs.len(), 3);
    }

    #[test]
    fn empty_history_works() {
        let msgs = MessageBuilder::new("system".into())
            .history(vec![])
            .context(&TextOnlyStrategy, "ctx".into())
            .build();

        // system + context = 2
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn preserves_message_order() {
        let msgs = MessageBuilder::new("system".into())
            .history(vec![
                Message::User("first".into()),
                Message::Assistant {
                    content: "reply".into(),
                    annotations: None,
                    reasoning_details: None,
                    images: Vec::new(),
                },
                Message::User("second".into()),
            ])
            .context(&TextOnlyStrategy, "ctx".into())
            .build();

        assert_eq!(msgs.len(), 5); // system + 3 history + context
        assert!(matches!(msgs[0], Message::System(_)));
        assert!(matches!(msgs[1], Message::User(_)));
        assert!(matches!(msgs[2], Message::Assistant { .. }));
        assert!(matches!(msgs[3], Message::User(_)));
        assert!(matches!(msgs[4], Message::User(_))); // context
    }
}
