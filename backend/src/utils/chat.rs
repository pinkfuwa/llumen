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

impl From<entity::ModeKind> for ChatMode {
    fn from(value: entity::ModeKind) -> Self {
        match value {
            entity::ModeKind::Normal => Self::Normal,
            entity::ModeKind::Search => Self::Search,
            entity::ModeKind::Research => Self::Research,
        }
    }
}

impl From<ChatMode> for entity::ModeKind {
    fn from(value: ChatMode) -> Self {
        match value {
            ChatMode::Normal => Self::Normal,
            ChatMode::Search => Self::Search,
            ChatMode::Research => Self::Research,
        }
    }
}
