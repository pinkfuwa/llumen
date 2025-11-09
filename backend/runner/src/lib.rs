//! # Lua Code Runner with Tree-Based State Caching
//!
//! This crate provides a safe and extensible interface for executing Lua code with
//! tree-based state management and caching.
//!
//! ## Overview
//!
//! The runner maintains a linear stack of executed commands and caches the results of the last two executions.
//! This provides simple caching for repeated exact command sequences while ensuring transparency—no errors for capacity issues.
//!
//! ## Execution Model
//!
//! The runner maintains a stack of previously executed commands and caches the serialized states from the last two executions.
//! For a new execution path (sequence of commands):
//! - If the path exactly matches the current stack, it returns the cached state (hit).
//! - Otherwise, it re-executes all commands from scratch in a fresh VM, updates the stack, and caches the new last two states.
//!
//! This supports linear histories (e.g., sequential messages in a chat) with exact matching for efficiency.
//! Branches or deletions require full re-execution, but there are no capacity limits or eviction errors.
//!
//! ## Caching Strategy
//!
//! ## Usage
//!
//! ```rust
//! use runner::{LuaRunner, LuaRunnerConfig, ExecutionPath};
//!
//! # async fn example() -> Result<(), runner::LuaRunnerError> {
//! let config = LuaRunnerConfig::default();
//! let mut runner = LuaRunner::new(config, None);
//!
//! // Execute first sequence
//! let path1 = vec!["x = 10"];
//! let result1 = runner.execute(&path1).await?;
//!
//! // Execute extended sequence (exact match + new command, caches last two)
//! let path2 = vec!["x = 10", "y = x + 5"];
//! let result2 = runner.execute(&path2).await?;
//!
//! // Re-execute exact path2 (cache hit, uses last state)
//! let result3 = runner.execute(&path2).await?;
//! assert!(result3.from_cache);
//!
//! // Execute different sequence (miss, re-executes from scratch)
//! let path4 = vec!["x = 10", "z = x * 2"];
//! let result4 = runner.execute(&path4).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Function Registration
//!
//! The `LuaRunner::new` constructor accepts an optional `custom_registrar` parameter of type `CustomRegistrar`. This is a boxed closure that takes `&mlua::Lua` and returns `Result<()>`. It allows library consumers to inject custom functions or modules (as Lua tables) into the global environment, enabling Lua code to access the consumer's API within the sandboxed execution.
//!
//! The registrar is invoked every time the runner creates a new Lua VM instance (e.g., for each command in an execution path). This ensures custom registrations are applied consistently across executions while allowing state isolation via cloning.
//!
//! ### Key Guidelines
//! - **Safety**: Custom functions must respect sandbox limits (e.g., no I/O, no unsafe operations). The runner does not validate registrations, so consumers are responsible for security.
//! - **Error Handling**: If the registrar returns an error, the runner will fail with `LuaRunnerError::InitializationError`.
//! - **Serialization**: Custom functions and userdata are not serialized in state snapshots (they appear as "function" or "userdata" in JSON). Only primitive values and tables are preserved across cache branches.
//! - **Dependencies**: Consumers must depend on `mlua` to create Lua functions/tables in the registrar.
//!
//! See the usage example above for how to register simple functions and modules. For complex APIs, structure them as tables to organize methods (e.g., `consumer_api.method()`).
//!
//! ## Error Handling
//!
//! All fallible operations return `Result<T, LuaRunnerError>`. Errors include:
//! - Syntax errors in Lua code
//! - Runtime errors during execution
//! - Resource limit violations
//! - No capacity-related errors; caching is transparent and unbounded for the stack/last two states.
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
//! ## Performance Considerations
//!
//! - State serialization uses JSON for primitive values and tables (functions/userdata excluded).
//! - Cache hits require exact path matching (O(n) comparison).
//! - Misses trigger full re-execution, so keep paths short.
//! - Memory usage is minimal: only stack of strings + two serialized states.

/// Maximum number of cached state nodes in the tree.
///
/// When this limit is exceeded, the least recently used nodes are evicted.
/// Each node contains a serialized Lua table state, so memory usage should
/// be considered when adjusting this value.

/// Default memory limit for Lua VM (in bytes).
pub const DEFAULT_MEMORY_LIMIT: usize = 64 * 1024 * 1024; // 64 MB

/// Default instruction count limit for Lua execution.
pub const DEFAULT_INSTRUCTION_LIMIT: usize = 10000;

mod config;
mod error;
mod runner;
pub mod tools;

pub use config::LuaRunnerConfig;
pub use error::LuaRunnerError;
pub use runner::LuaRunner;

/// Result type alias for runner operations.
pub type Result<T> = std::result::Result<T, LuaRunnerError>;

pub type ExecutionPath = Vec<String>;

/// Type alias for the custom registrar function.
/// This allows consumers to inject custom functions or modules into the Lua globals
/// before execution begins. The function receives a reference to the Lua instance
/// and should register any desired APIs in lua.globals(). It is invoked each time
/// a new Lua VM is created for execution.
pub type CustomRegistrar = Box<dyn Fn(&mlua::Lua) -> Result<()> + Send + Sync>;

/// Execution result containing output and metadata.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// The return value from the Lua code execution.
    pub output: String,

    /// Standard output captured during execution.
    pub stdout: String,

    /// Standard error captured during execution.
    pub stderr: String,

    /// Whether this result was retrieved from cache (exact path match).
    pub from_cache: bool,

    /// Number of instructions executed (None if from cache).
    pub instruction_count: Option<usize>,
}
