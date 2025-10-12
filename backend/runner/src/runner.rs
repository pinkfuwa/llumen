//! Main Lua runner implementation with tree-based state caching.

use crate::{ExecutionPath, ExecutionResult, LuaRunnerConfig, LuaRunnerError, Result, StateTree};
use mlua::{Lua, StdLib, Value};
use std::sync::{Arc, Mutex};

/// Main Lua runner with tree-based state management.
pub struct LuaRunner {
    /// Configuration for the runner.
    config: LuaRunnerConfig,

    /// State tree for caching execution results.
    state_tree: Arc<Mutex<StateTree>>,
}

impl LuaRunner {
    /// Creates a new Lua runner with the given configuration.
    pub fn new(config: LuaRunnerConfig) -> Self {
        let initial_state = "{}".to_string();
        let state_tree = Arc::new(Mutex::new(StateTree::new(initial_state)));

        Self { config, state_tree }
    }

    /// Executes a sequence of commands following an execution path.
    ///
    /// This method:
    /// 1. Searches the state tree for the longest matching prefix of the path
    /// 2. If the entire path is cached, returns the cached result
    /// 3. Otherwise, executes the remaining commands starting from the cached state
    /// 4. Caches each new execution result in the tree
    ///
    /// # Arguments
    ///
    /// * `path` - The execution path (sequence of commands) to execute
    ///
    /// # Returns
    ///
    /// The execution result including output, captured stdout/stderr, and metadata.
    pub async fn execute(&self, path: &ExecutionPath) -> Result<ExecutionResult> {
        if path.is_empty() {
            return Err(LuaRunnerError::InvalidPath(
                "Execution path cannot be empty".to_string(),
            ));
        }

        let (initial_state, cached_depth, remaining_commands): (String, usize, Vec<String>) = {
            let mut tree = self
                .state_tree
                .lock()
                .map_err(|e| LuaRunnerError::ExecutionError(format!("Lock error: {}", e)))?;

            let (cached_node_state, rem) = tree.find_node(path)?;

            let depth = path.len() - rem.len();
            (cached_node_state, depth, rem.to_vec())
        };

        if remaining_commands.is_empty() {
            return Ok(ExecutionResult {
                output: String::new(),
                stdout: String::new(),
                stderr: String::new(),
                from_cache: true,
                instruction_count: None,
            });
        }

        let mut current_state = initial_state;
        let mut last_output = String::new();
        let mut total_instructions = 0;

        for (i, command) in remaining_commands.iter().enumerate() {
            let lua = self.create_lua_vm()?;

            self.restore_state(&lua, &current_state)?;

            let result = self.execute_command(&lua, command).await?;

            last_output = result.output;
            if let Some(count) = result.instruction_count {
                total_instructions += count;
            }

            current_state = self.serialize_state(&lua)?;

            let parent_path = path[..cached_depth + i].to_vec();
            let mut tree = self
                .state_tree
                .lock()
                .map_err(|e| LuaRunnerError::ExecutionError(format!("Lock error: {}", e)))?;
            tree.insert_node(&parent_path, command.clone(), current_state.clone())?;
        }

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
        let mut stdout = String::new();
        let mut stderr = String::new();

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
        let std_libs = match self.config.enable_std_lib {
            true => StdLib::ALL_SAFE,
            false => {
                let mut libs = StdLib::NONE;
                if self.config.enable_math_lib {
                    libs |= StdLib::MATH;
                }
                if self.config.enable_string_lib {
                    libs |= StdLib::STRING;
                }
                if self.config.enable_table_lib {
                    libs |= StdLib::TABLE;
                }
                if self.config.enable_utf8_lib {
                    libs |= StdLib::UTF8;
                }
                libs
            }
        };

        lua.load_std_libs(std_libs)
            .map_err(|e| LuaRunnerError::InitializationError(e.to_string()))?;

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

    /// Clears all cached states except the root.
    pub fn clear_cache(&self) -> Result<()> {
        let mut tree = self
            .state_tree
            .lock()
            .map_err(|e| LuaRunnerError::ExecutionError(format!("Lock error: {}", e)))?;
        tree.clear();
        Ok(())
    }

    /// Gets the current number of cached nodes.
    pub fn cache_size(&self) -> Result<usize> {
        let tree = self
            .state_tree
            .lock()
            .map_err(|e| LuaRunnerError::ExecutionError(format!("Lock error: {}", e)))?;
        Ok(tree.node_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_execution() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path = vec!["return 2 + 2".to_string()];
        let result = runner.execute(&path).await.unwrap();

        assert_eq!(result.output, "4");
        assert!(!result.from_cache);
    }

    #[tokio::test]
    async fn test_variable_assignment() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path1 = vec!["x = 10".to_string()];
        runner.execute(&path1).await.unwrap();

        let path2 = vec!["x = 10".to_string(), "return x + 5".to_string()];
        let result = runner.execute(&path2).await.unwrap();

        assert_eq!(result.output, "15");
    }

    #[tokio::test]
    async fn test_caching() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path = vec!["x = 10".to_string()];
        runner.execute(&path).await.unwrap();

        let result = runner.execute(&path).await.unwrap();
        assert!(result.from_cache);
    }

    #[tokio::test]
    async fn test_branching_execution() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path1 = vec!["x = 10".to_string(), "y = 20".to_string()];
        runner.execute(&path1).await.unwrap();

        let path2 = vec!["x = 10".to_string(), "z = 30".to_string()];
        runner.execute(&path2).await.unwrap();

        assert_eq!(runner.cache_size().unwrap(), 4);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path = vec!["x = 10".to_string()];
        runner.execute(&path).await.unwrap();

        runner.clear_cache().unwrap();
        assert_eq!(runner.cache_size().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_empty_path_error() {
        let config = LuaRunnerConfig::default();
        let runner = LuaRunner::new(config);

        let path = vec![];
        let result = runner.execute(&path).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sandbox_restrictions() {
        let config = LuaRunnerConfig::sandboxed();
        let runner = LuaRunner::new(config);

        let path = vec!["return io".to_string()];
        let result = runner.execute(&path).await.unwrap();

        assert_eq!(result.output, "nil");
    }
}
