use super::runner;
use anyhow::Result;
use std::sync::Arc;

/// Lua REPL tool for code execution
pub struct LuaReplTool {
    runner: Arc<runner::LuaRunner>,
}

impl LuaReplTool {
    pub fn new() -> Self {
        let config = runner::LuaRunnerConfig::sandboxed();

        // Create runner with SQL and HTTP functions
        let runner = runner::LuaRunner::new(
            config,
            Some(Box::new(|lua| {
                let ctx = Arc::new(runner::tools::SqliteContext::new());
                runner::tools::register_sql_functions(lua, ctx)
                    .map_err(|e| runner::LuaRunnerError::InitializationError(e.to_string()))?;
                runner::tools::register_http_functions(lua)
                    .map_err(|e| runner::LuaRunnerError::InitializationError(e.to_string()))?;
                Ok(())
            })),
        );

        Self {
            runner: Arc::new(runner),
        }
    }

    /// Executes Lua code and returns the result
    pub async fn execute(&self, code: &str) -> Result<String> {
        let result = self.runner.execute(&[code]).await?;
        Ok(result.output)
    }

    /// Clears the Lua REPL state
    pub async fn clear(&self) -> Result<()> {
        Ok(self.runner.clear_cache().await?)
    }
}

pub fn get_lua_repl_def() -> crate::openrouter::Tool {
    crate::openrouter::Tool {
        name: "lua_repl".to_string(),
        description: "Execute lua code and do data analysis or calculation. If you want to see the output of a value, you should print it out with `print(...)`. This is visible to the user.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The lua code to execute to do further analysis or calculation."
                }
            },
            "required": ["code"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lua_repl() {
        let tool = LuaReplTool::new();
        let result = tool.execute("return 2 + 2").await.unwrap();
        assert_eq!(result, "4");
    }

    #[tokio::test]
    async fn test_lua_repl_error() {
        let tool = LuaReplTool::new();
        // Test that invalid Lua code returns an error
        let result = tool.execute("this is invalid lua code !!@@##").await;
        assert!(result.is_err());
    }
}
