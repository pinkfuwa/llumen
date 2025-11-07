use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum ChatMode {
    Normal,
    Search,
    Research,
}

impl From<protocol::ModeKind> for ChatMode {
    fn from(value: protocol::ModeKind) -> Self {
        match value {
            protocol::ModeKind::Normal => Self::Normal,
            protocol::ModeKind::Search => Self::Search,
            protocol::ModeKind::Research => Self::Research,
        }
    }
}

impl From<ChatMode> for protocol::ModeKind {
    fn from(value: ChatMode) -> Self {
        match value {
            ChatMode::Normal => Self::Normal,
            ChatMode::Search => Self::Search,
            ChatMode::Research => Self::Research,
        }
    }
}
