//! Configuration options for the Lua runner.

use crate::{DEFAULT_INSTRUCTION_LIMIT, DEFAULT_MEMORY_LIMIT};

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
    pub enable_math_lib: bool,

    /// Whether to enable the string library.
    pub enable_string_lib: bool,

    /// Whether to enable the table library.
    pub enable_table_lib: bool,

    /// Whether to enable the utf8 library.
    pub enable_utf8_lib: bool,

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
            enable_math_lib: true,
            enable_string_lib: true,
            enable_table_lib: true,
            enable_utf8_lib: true,
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
            enable_std_lib: false,
            ..Default::default()
        }
    }

    /// Creates a minimal configuration with all libraries disabled.
    pub fn minimal() -> Self {
        Self {
            enable_std_lib: false,
            enable_math_lib: false,
            enable_string_lib: false,
            enable_table_lib: false,
            enable_utf8_lib: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LuaRunnerConfig::default();
        assert_eq!(config.memory_limit, DEFAULT_MEMORY_LIMIT);
        assert_eq!(config.instruction_limit, DEFAULT_INSTRUCTION_LIMIT);
        assert!(config.enable_std_lib);
        assert!(config.capture_stdout);
    }

    #[test]
    fn test_sandboxed_config() {
        let config = LuaRunnerConfig::sandboxed();
        assert!(!config.enable_std_lib);
        assert!(config.enable_math_lib);
    }

    #[test]
    fn test_minimal_config() {
        let config = LuaRunnerConfig::minimal();
        assert!(!config.enable_std_lib);
        assert!(!config.enable_math_lib);
        assert!(!config.enable_string_lib);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LuaRunnerConfig::default()
            .with_memory_limit(1024)
            .with_instruction_limit(1000)
            .with_timeout(3000);

        assert_eq!(config.memory_limit, 1024);
        assert_eq!(config.instruction_limit, 1000);
        assert_eq!(config.timeout_ms, Some(3000));
    }
}
