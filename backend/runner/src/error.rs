//! Error types for the Lua runner.

use std::fmt;

/// Error type for Lua runner operations.
#[derive(Debug)]
pub enum LuaRunnerError {
    /// Error during Lua execution.
    ExecutionError(String),

    /// Lua syntax error.
    SyntaxError(String),

    /// Memory limit exceeded.
    MemoryLimitExceeded,

    /// Instruction count limit exceeded.
    InstructionLimitExceeded,

    /// Cache capacity exceeded.
    CacheCapacityExceeded,

    /// Invalid execution path.
    InvalidPath(String),

    /// State serialization/deserialization error.
    SerializationError(String),

    /// Lua VM initialization error.
    InitializationError(String),

    /// Operation not allowed in sandbox.
    SandboxViolation(String),
}

impl fmt::Display for LuaRunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionError(msg) => write!(f, "Lua execution error: {}", msg),
            Self::SyntaxError(msg) => write!(f, "Lua syntax error: {}", msg),
            Self::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            Self::InstructionLimitExceeded => write!(f, "Instruction count limit exceeded"),
            Self::CacheCapacityExceeded => write!(f, "Cache capacity exceeded"),
            Self::InvalidPath(msg) => write!(f, "Invalid execution path: {}", msg),
            Self::SerializationError(msg) => write!(f, "State serialization error: {}", msg),
            Self::InitializationError(msg) => write!(f, "Lua VM initialization error: {}", msg),
            Self::SandboxViolation(msg) => write!(f, "Sandbox violation: {}", msg),
        }
    }
}

impl std::error::Error for LuaRunnerError {}

impl From<mlua::Error> for LuaRunnerError {
    fn from(err: mlua::Error) -> Self {
        match err {
            mlua::Error::SyntaxError { message, .. } => Self::SyntaxError(message),
            mlua::Error::RuntimeError(msg) => Self::ExecutionError(msg),
            mlua::Error::MemoryError(msg) => {
                if msg.contains("limit") {
                    Self::MemoryLimitExceeded
                } else {
                    Self::ExecutionError(msg)
                }
            }
            _ => Self::ExecutionError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for LuaRunnerError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}
