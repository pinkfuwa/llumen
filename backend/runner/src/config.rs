//! Configuration options for the Lua runner.

use crate::{DEFAULT_INSTRUCTION_LIMIT, DEFAULT_MEMORY_LIMIT, DEEP_RESEARCH_MEMORY_LIMIT};

/// Configuration for the Lua runner.
#[derive(Debug, Clone)]
pub struct LuaRunnerConfig {
    /// Maximum memory that the Lua VM can allocate (in bytes).
    pub memory_limit: usize,

    /// Maximum number of instructions that can be executed.
    pub instruction_limit: usize,

    /// Whether to enable the Lua standard library.
    pub enable_std_lib: bool,

    /// Whether to enable the math library.
    pub sandboxed: bool,
    /// Whether to capture stdout during execution.
    pub capture_stdout: bool,

    /// Whether to capture stderr during execution.
    pub capture_stderr: bool,

    /// Timeout for script execution in milliseconds.
    pub timeout_ms: Option<u64>,
}

impl Default for LuaRunnerConfig {
    fn default() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            instruction_limit: DEFAULT_INSTRUCTION_LIMIT,
            enable_std_lib: true,
            sandboxed: false,
            capture_stdout: true,
            capture_stderr: true,
            timeout_ms: Some(5000), // 5 seconds default timeout
        }
    }
}

impl LuaRunnerConfig {
    /// Creates a new configuration with default sandboxing enabled.
    pub fn sandboxed() -> Self {
        Self {
            enable_std_lib: true,
            sandboxed: true,
            ..Default::default()
        }
    }

    /// Creates a configuration for deep research with higher limits.
    pub fn deep_research() -> Self {
        Self {
            memory_limit: DEEP_RESEARCH_MEMORY_LIMIT,
            instruction_limit: 1_000_000, // Higher limit for deep research
            enable_std_lib: true,
            sandboxed: true,
            capture_stdout: true,
            capture_stderr: true,
            timeout_ms: Some(60_000), // 60 seconds
        }
    }

    /// Creates a minimal configuration with all libraries disabled.
    pub fn minimal() -> Self {
        Self {
            enable_std_lib: true,
            sandboxed: false,
            ..Default::default()
        }
    }

    /// Sets the memory limit.
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Sets the instruction limit.
    pub fn with_instruction_limit(mut self, limit: usize) -> Self {
        self.instruction_limit = limit;
        self
    }

    /// Sets the timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Disables the timeout.
    pub fn without_timeout(mut self) -> Self {
        self.timeout_ms = None;
        self
    }
}
