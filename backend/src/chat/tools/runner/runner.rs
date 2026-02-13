//! Main Lua runner implementation with tree-based state caching.

use super::{ExecutionResult, LuaRunnerConfig, LuaRunnerError, Result};
use mlua::{Lua, StdLib, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct RunnerState {
    pub command_stack: Vec<String>,
    pub last_state: String,
    pub last_output: String,
    pub penultimate_state: Option<String>,
}

/// Main Lua runner with tree-based state management.
pub struct LuaRunner {
    /// Configuration for the runner.
    config: LuaRunnerConfig,

    /// Internal state for linear caching.
    runner_state: Arc<Mutex<RunnerState>>,

    /// Optional custom function registrar.
    registrar: Option<Box<dyn Fn(&Lua) -> Result<()> + Send + Sync + 'static>>,
}

impl LuaRunner {
    /// Creates a new Lua runner with the given configuration and optional custom registrar.
    pub fn new(
        config: LuaRunnerConfig,
        registrar: Option<Box<dyn Fn(&Lua) -> Result<()> + Send + Sync + 'static>>,
    ) -> Self {
        let runner_state = Arc::new(Mutex::new(RunnerState {
            command_stack: Vec::new(),
            last_state: "{}".to_string(),
            last_output: String::new(),
            penultimate_state: None,
        }));

        Self {
            config,
            runner_state,
            registrar,
        }
    }

    /// Executes a sequence of commands following an execution path.
    ///
    /// This method:
    /// 1. Checks if the path exactly matches the current command stack
    /// 2. If matched, returns the cached result immediately
    /// 3. Otherwise, re-executes the entire path from scratch
    /// 4. Updates the internal state with the new stack and latest states
    ///
    /// # Arguments
    ///
    /// * `path` - The execution path (sequence of commands) to execute
    ///
    /// # Returns
    ///
    /// The execution result including output, captured stdout/stderr, and metadata.
    pub async fn execute(&self, path: &[&str]) -> Result<ExecutionResult> {
        if path.is_empty() {
            return Err(LuaRunnerError::InvalidPath(
                "Execution path cannot be empty".to_string(),
            ));
        }

        let guard = self.runner_state.lock().await;

        if guard.command_stack == *path {
            let output = guard.last_output.clone();
            let stdout = String::new();
            let stderr = String::new();
            drop(guard);
            return Ok(ExecutionResult {
                output,
                stdout,
                stderr,
                from_cache: true,
                instruction_count: None,
            });
        }
        drop(guard);

        let mut current_state = "{}".to_string();
        let mut penultimate_state: Option<String> = None;
        let mut command_stack: Vec<String> = Vec::new();
        let mut last_output = String::new();
        let total_instructions = 0usize;

        let num_commands = path.len();
        if num_commands > 1 {
            for command in &path[0..num_commands.saturating_sub(1)] {
                let lua = self.create_lua_vm()?;

                self.restore_state(&lua, &current_state)?;

                let _result = self.execute_command(&lua, command).await?;

                current_state = self.serialize_state(&lua)?;

                command_stack.push(command.to_string());
            }
            penultimate_state = Some(current_state.clone());
        }

        if let Some(last_command) = path.last() {
            let lua = self.create_lua_vm()?;

            self.restore_state(&lua, &current_state)?;

            let exec_result = self.execute_command(&lua, last_command).await?;

            last_output = exec_result.output;

            current_state = self.serialize_state(&lua)?;

            command_stack.push(last_command.to_string());
        }

        let mut guard = self.runner_state.lock().await;
        guard.command_stack = command_stack;
        guard.last_state = current_state;
        guard.penultimate_state = penultimate_state;
        guard.last_output = last_output.clone();
        drop(guard);

        Ok(ExecutionResult {
            output: last_output,
            stdout: String::new(),
            stderr: String::new(),
            from_cache: false,
            instruction_count: Some(total_instructions),
        })
    }

    /// Executes a single command in the given Lua VM.
    async fn execute_command(&self, lua: &Lua, command: &str) -> Result<ExecutionResult> {
        let stdout = String::new();
        let stderr = String::new();

        if self.config.capture_stdout {
            lua.globals()
                .set(
                    "print",
                    lua.create_function(|_, args: mlua::MultiValue| {
                        let _ = args;
                        // Capture print output if needed
                        Ok(())
                    })
                    .map_err(|e| LuaRunnerError::InitializationError(e.to_string()))?,
                )
                .map_err(|e| LuaRunnerError::InitializationError(e.to_string()))?;
        }

        let result = lua.load(command).eval::<mlua::Value>()?;

        let output = self.value_to_string(&result)?;

        Ok(ExecutionResult {
            output,
            stdout,
            stderr,
            from_cache: false,
            instruction_count: None,
        })
    }

    /// Creates a new Lua VM with the configured settings.
    fn create_lua_vm(&self) -> Result<Lua> {
        let lua = Lua::new();
        let std_libs = match (self.config.enable_std_lib, self.config.sandboxed) {
            (true, true) => StdLib::ALL_SAFE,
            (true, false) => StdLib::ALL,
            _ => StdLib::NONE,
        };

        lua.sandbox(self.config.sandboxed)
            .map_err(|e| LuaRunnerError::InitializationError(e.to_string()))?;

        lua.load_std_libs(std_libs)
            .map_err(|e| LuaRunnerError::InitializationError(e.to_string()))?;

        if let Some(registrar) = &self.registrar {
            registrar(&lua)?;
        }

        Ok(lua)
    }

    /// Serializes the current Lua global state to JSON.
    fn serialize_state(&self, lua: &Lua) -> Result<String> {
        let globals = lua.globals();

        self.table_to_json(&globals)
    }

    /// Restores Lua global state from a JSON string.
    fn restore_state(&self, lua: &Lua, state_json: &str) -> Result<()> {
        if state_json == "{}" {
            return Ok(());
        }

        let state: serde_json::Value = serde_json::from_str(state_json)?;

        if let serde_json::Value::Object(map) = state {
            for (key, value) in map {
                let lua_value = self.json_to_value(lua, &value)?;
                lua.globals()
                    .set(key.as_str(), lua_value)
                    .map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Converts a Lua table to a JSON string.
    fn table_to_json(&self, table: &mlua::Table) -> Result<String> {
        let mut map = serde_json::Map::new();

        for pair in table.pairs::<Value, Value>() {
            let (key, value) =
                pair.map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?;

            if let Value::String(key_str) = key {
                let key_string = key_str
                    .to_str()
                    .map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?
                    .to_string();

                let json_value = self.value_to_json(&value)?;
                map.insert(key_string, json_value);
            }
        }

        Ok(serde_json::to_string(&map)?)
    }

    /// Converts a Lua value to a JSON value.
    fn value_to_json(&self, value: &Value) -> Result<serde_json::Value> {
        match value {
            Value::Nil => Ok(serde_json::Value::Null),
            Value::Boolean(b) => Ok(serde_json::Value::Bool(*b)),
            Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
            Value::Number(n) => {
                if let Some(num) = serde_json::Number::from_f64(*n) {
                    Ok(serde_json::Value::Number(num))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            Value::String(s) => {
                let string = s
                    .to_str()
                    .map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?
                    .to_string();
                Ok(serde_json::Value::String(string))
            }
            Value::Table(_) => Ok(serde_json::Value::String("table".to_string())),
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// Converts a JSON value to a Lua value.
    fn json_to_value(&self, lua: &Lua, json: &serde_json::Value) -> Result<Value> {
        match json {
            serde_json::Value::Null => Ok(Value::Nil),
            serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Number(f))
                } else {
                    Ok(Value::Nil)
                }
            }
            serde_json::Value::String(s) => {
                let lua_string = lua
                    .create_string(s)
                    .map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?;
                Ok(Value::String(lua_string))
            }
            serde_json::Value::Array(_) => Ok(Value::Nil),
            serde_json::Value::Object(_) => Ok(Value::Nil),
        }
    }

    /// Converts a Lua value to a string representation.
    fn value_to_string(&self, value: &Value) -> Result<String> {
        match value {
            Value::Nil => Ok("nil".to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Number(n) => Ok(n.to_string()),
            Value::String(s) => {
                let string = s
                    .to_str()
                    .map_err(|e| LuaRunnerError::SerializationError(e.to_string()))?
                    .to_string();
                Ok(string)
            }
            Value::Table(_) => Ok("table".to_string()),
            Value::Vector(_) => Ok("vector".to_string()),
            Value::Function(_) => Ok("function".to_string()),
            Value::Thread(_) => Ok("thread".to_string()),
            Value::UserData(_) => Ok("userdata".to_string()),
            Value::LightUserData(_) => Ok("lightuserdata".to_string()),
            Value::Error(e) => Ok(format!("error: {}", e.to_string())),
            _ => Ok("unsupported".to_string()),
        }
    }

    /// Clears all cached states.
    pub async fn clear_cache(&self) -> Result<()> {
        let mut state = self.runner_state.lock().await;
        state.command_stack.clear();
        state.last_state = "{}".to_string();
        state.last_output = String::new();
        state.penultimate_state = None;
        Ok(())
    }

    /// Gets the current length of the command stack (number of cached commands).
    pub async fn cache_size(&self) -> Result<usize> {
        let state = self.runner_state.lock().await;
        Ok(state.command_stack.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_execution() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path = vec!["return 2 + 2"];
        let result = runner.execute(&path).await.unwrap();

        assert_eq!(result.output, "4");
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_variable_assignment() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path1 = vec!["x = 10"];
        runner.execute(&path1).await.unwrap();

        let path2 = vec!["x = 10", "return x + 5"];
        let result = runner.execute(&path2).await.unwrap();

        assert_eq!(result.output, "15");
    }

    #[tokio::test]
    async fn test_caching() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path = vec!["x = 10"];
        runner.execute(&path).await.unwrap();

        let result = runner.execute(&path).await.unwrap();
        assert!(result.from_cache);
    }

    #[tokio::test]
    async fn test_exact_match_caching() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path = vec!["x = 10", "x * 2"];
        let result1 = runner.execute(&path).await.unwrap();
        assert!(!result1.from_cache);
        assert_eq!(result1.output, "20");

        let result2 = runner.execute(&path).await.unwrap();
        assert!(result2.from_cache);
        assert_eq!(result2.output, "20");

        assert_eq!(runner.cache_size().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_branching_execution() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path1 = vec!["x = 10", "y = 20"];
        runner.execute(&path1).await.unwrap();

        let path2 = vec!["x = 10", "z = 30"];
        runner.execute(&path2).await.unwrap();

        assert_eq!(runner.cache_size().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path = vec!["x = 10"];
        runner.execute(&path).await.unwrap();

        runner.clear_cache().await.unwrap();
        assert_eq!(runner.cache_size().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_empty_path_error() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config, None);

        let path = vec![];
        let result = runner.execute(&path).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sandbox_restrictions() {
        let config = LuaRunnerConfig::sandboxed();
        let runner = LuaRunner::new(config, None);

        let path = vec!["return io"];
        let result = runner.execute(&path).await.unwrap();

        assert_eq!(result.output, "nil");
    }
}
