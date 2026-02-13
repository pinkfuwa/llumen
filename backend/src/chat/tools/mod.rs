//! A collection of tools
//!
//! Tool contain tool prompt, a async method for solving tool output
//!
//! If the method return Err, step current completion with error message displayed to user
//!
//! If the error is expected to be seen by llumen(LLM), return Ok(string)
//!
//! Tool cannot manipulate and direct output, but subagent can

use std::sync::Arc;

// TODO: make duckduckgo(web_search) tool stateful(reuse same reqwest client with flyweight)
pub(crate) mod crawl;
pub(crate) mod lua;
#[allow(unused)]
pub(crate) mod runner;
pub(crate) mod web_search;

pub(crate) use crawl::{CrawlTool, get_crawl_tool_def};
pub(crate) use lua::{LuaReplTool, get_lua_repl_def};
pub(crate) use web_search::{WebSearchTool, get_web_search_tool_def};

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
}

impl Tools {
    /// Creates the tools collection with default implementations.
    pub fn new() -> Self {
        Self {
            web_search: Arc::new(WebSearchTool::new()),
            crawl: Arc::new(CrawlTool::new()),
            lua_repl: Arc::new(LuaReplTool::new()),
        }
    }

    /// Returns tool definitions for search mode.
    ///
    /// Includes: web_search, crawl
    pub fn for_search_mode(&self) -> Vec<crate::openrouter::Tool> {
        vec![get_web_search_tool_def(), get_crawl_tool_def()]
    }

    /// Returns tool definitions for normal mode.
    ///
    /// Currently: empty (normal mode has no tools)
    pub fn for_normal_mode(&self) -> Vec<crate::openrouter::Tool> {
        vec![]
    }

    /// Returns tool definitions for deep research mode.
    ///
    /// Always includes crawl, optionally includes web_search if need_search is true.
    pub fn for_deep_mode(&self, need_search: bool) -> Vec<crate::openrouter::Tool> {
        let mut tools = vec![get_crawl_tool_def()];
        if need_search {
            tools.push(get_web_search_tool_def());
        }
        tools
    }
}

impl Default for Tools {
    fn default() -> Self {
        Self::new()
    }
}
