//! A collection of tools
//!
//! Tool contain tool prompt, a async method for solving tool output
//!
//! If the method return Err, step current completion with error message
//! displayed to user
//!
//! If the error is expected to be seen by llumen(LLM), return Ok(string)
//!
//! Tool cannot manipulate and direct output, but subagent can

use std::sync::Arc;

use protocol::ModeKind;

// TODO: make duckduckgo(web_search) tool stateful(reuse same reqwest client
// with flyweight)
pub(crate) mod crawl;
pub(crate) mod lua;
#[allow(unused)]
pub(crate) mod runner;
pub(crate) mod web_search;

pub(crate) use crawl::{CrawlTool, get_crawl_tool_def};
pub(crate) use lua::{LuaReplTool, get_lua_repl_def};
pub(crate) use web_search::{WebSearchTool, get_web_search_tool_def};

use crate::mcp::manager::McpClientManager;

/// MCP tool name prefix: `mcp_{server_id}_{original_name}`
const MCP_PREFIX: &str = "mcp_";

/// Rich content from an MCP tool call (images, resources) for frontend display.
#[derive(Debug, Clone)]
pub enum McpRichContent {
    Image {
        data: String,
        mime_type: String,
    },
    Resource {
        uri: String,
        mime_type: Option<String>,
        text: Option<String>,
    },
}

/// Result of a tool execution: text for LLM + optional rich content for
/// frontend.
pub struct ToolOutput {
    /// Text to send to LLM as the tool result.
    pub text: String,
    /// Rich content parts to stream to the frontend (images, resources).
    pub rich: Vec<McpRichContent>,
}

/// Collection of all available tools in llumen.
///
/// **Purpose**: Group tools in one place instead of scattering them in Context.
///
/// **Benefits**:
/// - Easy to see what tools exist
/// - Easy to add/remove tools
/// - Easy to filter tools per mode
/// - Testable: can create Tools with mocks
pub struct Tools {
    pub(crate) web_search: Arc<WebSearchTool>,
    pub(crate) crawl: Arc<CrawlTool>,
    pub(crate) lua_repl: Arc<LuaReplTool>,
    pub(crate) mcp: Arc<McpClientManager>,
}

impl Tools {
    /// Creates the tools collection with default implementations.
    pub fn new(mcp: Arc<McpClientManager>) -> Self {
        Self {
            web_search: Arc::new(WebSearchTool::new()),
            crawl: Arc::new(CrawlTool::new()),
            lua_repl: Arc::new(LuaReplTool::new()),
            mcp,
        }
    }

    /// Returns tool definitions for search mode (built-in + MCP).
    pub async fn for_search_mode(&self) -> Vec<crate::openrouter::Tool> {
        let mut tools = vec![get_web_search_tool_def(), get_crawl_tool_def()];
        tools.extend(self.mcp_tools_for_mode(ModeKind::Search).await);
        tools
    }

    /// Returns tool definitions for normal mode (MCP only).
    pub async fn for_normal_mode(&self) -> Vec<crate::openrouter::Tool> {
        self.mcp_tools_for_mode(ModeKind::Normal).await
    }

    /// Returns tool definitions for deep research mode (built-in + MCP).
    pub async fn for_deep_mode(&self, need_search: bool) -> Vec<crate::openrouter::Tool> {
        let mut tools = vec![get_crawl_tool_def()];
        if need_search {
            tools.push(get_web_search_tool_def());
        }
        tools.extend(self.mcp_tools_for_mode(ModeKind::Research).await);
        tools
    }

    /// Convert MCP tools from a mode into openrouter Tool format.
    async fn mcp_tools_for_mode(&self, mode: ModeKind) -> Vec<crate::openrouter::Tool> {
        let mcp_tools = self.mcp.list_tools_for_mode(mode).await;
        mcp_tools
            .into_iter()
            .map(|(server_id, tool)| {
                let prefixed_name = format!("{MCP_PREFIX}{server_id}_{}", tool.name);
                crate::openrouter::Tool {
                    name: prefixed_name,
                    description: tool.description.map(|d| d.to_string()).unwrap_or_default(),
                    schema: serde_json::Value::Object(tool.input_schema.as_ref().clone()),
                }
            })
            .collect()
    }

    /// Execute a tool call by name. Routes to built-in or MCP server.
    /// Returns structured output: text for LLM + rich content for frontend.
    pub async fn execute_tool(&self, tool_name: &str, args: &str) -> ToolOutput {
        if let Some(rest) = tool_name.strip_prefix(MCP_PREFIX) {
            return self.execute_mcp_tool(rest, args).await;
        }
        ToolOutput {
            text: self.execute_builtin_tool(tool_name, args).await,
            rich: Vec::new(),
        }
    }

    /// Route to MCP: parse `{server_id}_{tool_name}` and call.
    async fn execute_mcp_tool(&self, rest: &str, args: &str) -> ToolOutput {
        let (server_id_str, tool_name) = match rest.split_once('_') {
            Some(pair) => pair,
            None => {
                return ToolOutput {
                    text: format!("Error: invalid MCP tool name format: {rest}"),
                    rich: Vec::new(),
                };
            }
        };
        let server_id: i32 = match server_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                return ToolOutput {
                    text: format!("Error: invalid MCP server id: {server_id_str}"),
                    rich: Vec::new(),
                };
            }
        };
        let parsed_args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(args).unwrap_or_default();
        self.mcp.call_tool(server_id, tool_name, parsed_args).await
    }

    /// Execute a built-in tool by name.
    async fn execute_builtin_tool(&self, tool_name: &str, args: &str) -> String {
        match tool_name {
            "web_search_tool" => {
                #[derive(serde::Deserialize)]
                struct A {
                    query: String,
                }
                let Some(a) = serde_json::from_str::<A>(args).ok() else {
                    return "Invalid arguments for web_search_tool".to_string();
                };
                match self.web_search.search(&a.query).await {
                    Ok(results) => {
                        let mut out = String::new();
                        for (i, r) in results.iter().enumerate().take(10) {
                            out.push_str(&format!(
                                "{}. [{}]({})\n   {}\n\n",
                                i + 1,
                                r.title,
                                r.url,
                                r.description
                            ));
                        }
                        if out.is_empty() {
                            out = "No search results found.".to_string();
                        }
                        out
                    }
                    Err(e) => {
                        log::warn!("Web search error: {}", e);
                        format!("Error: {}", e)
                    }
                }
            }
            "crawl_tool" => {
                #[derive(serde::Deserialize)]
                struct A {
                    url: String,
                }
                let Some(a) = serde_json::from_str::<A>(args).ok() else {
                    return "Invalid arguments for crawl_tool".to_string();
                };
                match self.crawl.crawl(&a.url).await {
                    Ok(content) => content,
                    Err(e) => {
                        log::warn!("Crawl error for URL '{}': {}", a.url, e);
                        format!("Error: {}", e)
                    }
                }
            }
            "lua_repl" => {
                #[derive(serde::Deserialize)]
                struct A {
                    code: String,
                }
                let Some(a) = serde_json::from_str::<A>(args).ok() else {
                    return "Invalid arguments for lua_repl".to_string();
                };
                match self.lua_repl.execute(&a.code).await {
                    Ok(result) => result,
                    Err(e) => {
                        log::warn!("Lua execution error: {}", e);
                        format!("Error: {}", e)
                    }
                }
            }
            _ => format!("Unknown tool: {}", tool_name),
        }
    }
}
