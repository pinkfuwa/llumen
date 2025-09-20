mod set;
mod store;
mod tool;

pub use set::*;
pub use store::*;
pub use tool::*;

use crate::tool_set;

pub mod wttr;
pub mod nearbyplace;
pub mod mail;
pub mod rss;

pub const NORMAL: ToolSet = tool_set![];
pub const SEARCH: ToolSet = tool_set![wttr::Wttr];
pub const AGENT: ToolSet = tool_set![wttr::Wttr, nearbyplace::NearByPlace, mail::RecentMail, mail::ReplyMail, mail::SendMail, mail::GetMailContent, rss::RssSearch];
pub const RESEARCH: ToolSet = tool_set![];
