use crate::openrouter;

/// Execution represents what to send to the LLM for one completion call.
///
/// **Purpose**: Immutable snapshot of execution input.
///
/// **Lifetime**: Created before each LLM call, consumed by that call.
/// In tool call loops, a new Execution is built for each iteration.
///
/// **Why separate from CompletionSession?**
/// - Session = per-request state (DB entities, publisher, usage tracking)
/// - Execution = per-LLM-call data (messages, tools, model config)
/// - Same session can create multiple Executions (tool call loop)
///
/// **Benefits**:
/// - Easy to test: construct Execution, inspect it
/// - Easy to log: see exactly what's being sent to LLM
/// - Clear separation: WHAT (data) vs HOW (strategy)
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
    pub fn new(
        messages: Vec<openrouter::Message>,
        options: openrouter::CompletionOption,
    ) -> Self {
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
