mod set;
mod store;
mod tool;

pub use set::*;
pub use store::*;
pub use tool::*;

use crate::tool_set;

pub mod wttr;

pub const NORMAL: ToolSet = tool_set![wttr::Wttr];
pub const SEARCH: ToolSet = tool_set![wttr::Wttr];
pub const AGENT: ToolSet = tool_set![wttr::Wttr];
pub const RESEARCH: ToolSet = tool_set![wttr::Wttr];
