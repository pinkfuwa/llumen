use crate::openrouter::{self, StreamCompletionResp};

use super::channel::Mergeable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub name: String,
    pub id: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Token {
    User(String),
    Assistant(String),
    Tool {
        name: String,
        args: String,
        id: String,
    },
    ToolToken(String),
    ToolResult(String),
    Reasoning(String),
    Empty,
    ResearchPlan(String),
    ResearchStep(String),
    ResearchReport(String),
    Error(String),
    Complete {
        message_id: i32,
        cost: f32,
        token: i32,
    },
    Title(String),
    Start {
        id: i32,
        user_msg_id: i32,
    },
}

impl Mergeable for Token {
    fn merge(&mut self, other: Self) -> Option<Self> {
        match (self, other) {
            (Token::Assistant(s1), Token::Assistant(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::Reasoning(s1), Token::Reasoning(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::ToolToken(s1), Token::ToolToken(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::ResearchPlan(s1), Token::ResearchPlan(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::ResearchStep(s1), Token::ResearchStep(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::ResearchReport(s1), Token::ResearchReport(s2)) => {
                s1.push_str(&s2);
                None
            }
            (_, other) => Some(other),
        }
    }

    fn len(&self) -> usize {
        match self {
            Token::User(s)
            | Token::Assistant(s)
            | Token::ToolToken(s)
            | Token::Reasoning(s)
            | Token::ResearchPlan(s)
            | Token::ResearchStep(s)
            | Token::ResearchReport(s)
            | Token::Error(s) => s.len(),
            Token::Tool { .. }
            | Token::ToolResult { .. }
            | Token::Empty
            | Token::Start { .. }
            | Token::Complete { .. }
            | Token::Title { .. } => 1,
        }
    }

    fn slice(&self, r: std::ops::Range<usize>) -> Option<Self> {
        match self {
            Token::User(s) => Some(Token::User(s[r].to_string())),
            Token::Assistant(s) => Some(Token::Assistant(s[r].to_string())),
            Token::ToolToken(s) => Some(Token::ToolToken(s[r].to_string())),
            Token::Reasoning(s) => Some(Token::Reasoning(s[r].to_string())),
            Token::ResearchPlan(s) => Some(Token::ResearchPlan(s[r].to_string())),
            Token::ResearchStep(s) => Some(Token::ResearchStep(s[r].to_string())),
            Token::ResearchReport(s) => Some(Token::ResearchReport(s[r].to_string())),
            Token::Error(s) => Some(Token::Error(s[r].to_string())),
            x if r.start == 0 => Some(x.clone()),
            _ => None,
        }
    }
}

impl From<openrouter::StreamCompletionResp> for Token {
    fn from(resp: openrouter::StreamCompletionResp) -> Self {
        match resp {
            StreamCompletionResp::ReasoningToken(reasoning) => Token::Reasoning(reasoning),
            StreamCompletionResp::ResponseToken(content) => Token::Assistant(content),
            StreamCompletionResp::ToolCall { name, args, id } => Token::Tool { name, args, id },
            StreamCompletionResp::ToolToken(token) => Token::ToolToken(token),
            _ => Token::Empty,
        }
    }
}
