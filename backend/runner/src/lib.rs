//! # Lua Code Runner with Tree-Based State Caching
//!
//! This crate provides a safe and extensible interface for executing Lua code with
//! tree-based state management and caching.
//!
//! ## Overview
//!
//! The runner maintains a tree structure where each node represents a Lua execution state.
//! This allows for efficient branching and caching of execution histories, enabling
//! features like undo/redo and alternative execution paths.
//!
//! ## Execution Model
//!
//! Consider the following execution sequence:
//!
//! 1. User runs `function_a()` → creates message 1
//! 2. Assistant executes `function_a()` → State A
//! 3. User runs `function_b()` → creates message 2
//! 4. Assistant executes `function_b()` → State B (child of State A)
//! 5. User deletes message 2
//! 6. User runs `function_c()` → creates message 3
//! 7. Assistant executes `function_c()` → State C (another child of State A)
//!
//! This results in a tree structure:
//!
//! ```text
//! Root (initial state)
//!   └─ function_a() → State A
//!       ├─ function_b() → State B
//!       └─ function_c() → State C
//! ```
//!
//! ## Caching Strategy
//!
//! - Each execution clones the parent state's Lua table before mutation
//! - The command and resulting state are stored in a tree node
//! - Future executions provide a path (sequence of commands)
//! - The runner searches the tree for matching paths
//! - If found, returns the cached state
//! - If not found, executes from the last cached point and stores the result
//! - Cache capacity is limited by `MAX_CACHE_NODES`
//!
//! ## Usage
//!
//! ```rust
//! use runner::{LuaRunner, LuaRunnerConfig, ExecutionPath};
//!
//! # async fn example() -> Result<(), runner::LuaRunnerError> {
//! let config = LuaRunnerConfig::default();
//! let mut runner = LuaRunner::new(config);
//!
//! // Execute first command
//! let path1 = vec!["x = 10".to_string()];
//! let result1 = runner.execute(&path1).await?;
//!
//! // Execute second command (builds on first)
//! let path2 = vec!["x = 10".to_string(), "y = x + 5".to_string()];
//! let result2 = runner.execute(&path2).await?;
//!
//! // Execute alternative path (reuses first command from cache)
//! let path3 = vec!["x = 10".to_string(), "z = x * 2".to_string()];
//! let result3 = runner.execute(&path3).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All fallible operations return `Result<T, LuaRunnerError>`. Errors include:
//! - Syntax errors in Lua code
//! - Runtime errors during execution
//! - Resource limit violations
//! - Cache capacity exceeded
//!
//! ## Safety and Sandboxing
//!
//! The Lua environment is sandboxed to prevent:
//! - File system access
//! - Network operations
//! - System calls
//! - Excessive memory usage
//! - Infinite loops (via instruction count limits)
//!
//! ## Configuration
//!
//! Customize the runner behavior through `LuaRunnerConfig`:
//! - Memory limits
//! - Instruction count limits
//! - Allowed Lua standard libraries
//! - Custom function registrations
//!
//! ## Tree Structure
//!
//! The internal tree structure (`StateTree`) contains:
//! - `StateNode`: Represents a single execution state
//!   - Command executed to reach this state
//!   - Serialized Lua table state
//!   - Children nodes (subsequent executions)
//!   - Metadata (creation time, access count)
//!
//! ## Cache Management
//!
//! - Maximum cache size: `MAX_CACHE_NODES` (default: 1000)
//! - Eviction policy: Least Recently Used (LRU)
//! - Each node tracks last access time for eviction
//!
//! ## Performance Considerations
//!
//! - Table cloning uses mlua's built-in serialization/deserialization
//! - Cache lookups are O(n) where n is the path length
//! - Memory usage scales with number of cached states
//! - Consider adjusting `MAX_CACHE_NODES` based on available memory

/// Maximum number of cached state nodes in the tree.
///
/// When this limit is exceeded, the least recently used nodes are evicted.
/// Each node contains a serialized Lua table state, so memory usage should
/// be considered when adjusting this value.
pub const MAX_CACHE_NODES: usize = 1000;

/// Default memory limit for Lua VM (in bytes).
pub const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024; // 64 MB

/// Default instruction count limit for Lua execution.
pub const DEFAULT_INSTRUCTION_LIMIT: usize = 10_000_000;

mod config;
mod error;
mod runner;
mod state_tree;

pub use config::LuaRunnerConfig;
pub use error::LuaRunnerError;
pub use runner::LuaRunner;
pub use state_tree::{ExecutionPath, StateNode, StateTree};

/// Result type alias for runner operations.
pub type Result<T> = std::result::Result<T, LuaRunnerError>;

/// Execution result containing output and metadata.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// The return value from the Lua code execution.
    pub output: String,

    /// Standard output captured during execution.
    pub stdout: String,

    /// Standard error captured during execution.
    pub stderr: String,

    /// Whether this result was retrieved from cache.
    pub from_cache: bool,

    /// Number of instructions executed (None if from cache).
    pub instruction_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CACHE_NODES, 1000);
        assert_eq!(DEFAULT_MEMORY_LIMIT, 64 * 1024 * 1024);
        assert_eq!(DEFAULT_INSTRUCTION_LIMIT, 10_000_000);
    }
}
