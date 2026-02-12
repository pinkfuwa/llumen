//! A collection of tools
//!
//! Tool contain tool prompt, a async method for solving tool output
//!
//! If the method return Err, step current completion with error message displayed to user
//!
//! If the error is expected to be seen by llumen(LLM), return Ok(string)
//!
//! Tool cannot manipulate and direct output, but subagent can

// TODO: make duckduckgo(web_search) tool stateful(reuse same reqwest client with flyweight)
pub(crate) mod crawl;
pub(crate) mod lua;
pub(crate) mod web_search;

pub(crate) use crawl::{CrawlTool, get_crawl_tool_def};
pub(crate) use lua::{LuaReplTool, get_lua_repl_def};
pub(crate) use web_search::{WebSearchTool, get_web_search_tool_def};
