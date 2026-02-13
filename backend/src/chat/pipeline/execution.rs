use crate::openrouter;

/// Execution represents what to send to the LLM for one completion call.
///
/// Immutable snapshot of execution input.
pub struct Execution {
    /// The messages to send to the LLM (system prompt + history + context)
    pub messages: Vec<openrouter::Message>,

    /// The tools available for this execution
    pub tools: Vec<openrouter::Tool>,

    /// The model configuration (temperature, max_tokens, etc)
    pub options: openrouter::CompletionOption,
}

impl Execution {
    /// Creates a new execution with the given messages and options.
    pub fn new(messages: Vec<openrouter::Message>, options: openrouter::CompletionOption) -> Self {
        Self {
            messages,
            tools: Vec::new(),
            options,
        }
    }

    /// Adds tools to this execution (builder pattern).
    pub fn with_tools(mut self, tools: Vec<openrouter::Tool>) -> Self {
        self.tools = tools;
        self
    }
}
