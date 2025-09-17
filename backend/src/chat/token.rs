use crate::openrouter::{self, StreamCompletionResp};

use super::channel::Mergeable;
use entity::{ChunkKind, chunk};
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
    Message(String),
    Tool {
        name: String,
        args: String,
        id: String,
    },
    ToolResult(String),
    Reasoning(String),
    Empty,
    Plan(String),
    Step(String),
    Report(String),
    Error(String),
    Complete {
        message_id: i32,
        chunk_ids: Vec<i32>,
    },
}

impl Mergeable for Token {
    fn merge(self, other: Self) -> (Self, Option<Self>) {
        match (self, other) {
            (Token::User(s1), Token::User(s2)) => (Token::User(s1 + &s2), None),
            (Token::Message(s1), Token::Message(s2)) => (Token::Message(s1 + &s2), None),
            (Token::Tool { name, args, id }, Token::ToolResult(res)) => {
                let tool_call_info = ToolCallInfo {
                    name,
                    id,
                    input: args,
                    output: Some(res),
                };
                let content = serde_json::to_string(&tool_call_info).unwrap();
                (Token::ToolResult(content), None)
            }
            (Token::Reasoning(s1), Token::Reasoning(s2)) => (Token::Reasoning(s1 + &s2), None),
            (Token::Plan(s1), Token::Plan(s2)) => (Token::Plan(s1 + &s2), None),
            (Token::Step(s1), Token::Step(s2)) => (Token::Step(s1 + &s2), None),
            (Token::Report(s1), Token::Report(s2)) => (Token::Report(s1 + &s2), None),
            (Token::Error(s1), Token::Error(s2)) => (Token::Error(s1 + &s2), None),
            (s1, s2) => (s1, Some(s2)),
        }
    }

    fn len(&self) -> usize {
        match self {
            Token::User(s)
            | Token::Message(s)
            | Token::Reasoning(s)
            | Token::Plan(s)
            | Token::Step(s)
            | Token::Report(s)
            | Token::Error(s) => s.len(),
            Token::Tool { .. } | Token::ToolResult(_) | Token::Empty | Token::Complete { .. } => 0,
        }
    }

    fn split_end(&self, split: usize) -> Option<Self> {
        match self {
            Token::User(s) => Some(Token::User(s[split..].to_string())),
            Token::Message(s) => Some(Token::Message(s[split..].to_string())),
            Token::Reasoning(s) => Some(Token::Reasoning(s[split..].to_string())),
            Token::Plan(s) => Some(Token::Plan(s[split..].to_string())),
            Token::Step(s) => Some(Token::Step(s[split..].to_string())),
            Token::Report(s) => Some(Token::Report(s[split..].to_string())),
            Token::Error(s) => Some(Token::Error(s[split..].to_string())),
            _ => None,
        }
    }

    fn split_start(&self, split: usize) -> Self {
        match self {
            Token::User(s) => Token::User(s[..split].to_string()),
            Token::Message(s) => Token::Message(s[..split].to_string()),
            Token::Reasoning(s) => Token::Reasoning(s[..split].to_string()),
            Token::Plan(s) => Token::Plan(s[..split].to_string()),
            Token::Step(s) => Token::Step(s[..split].to_string()),
            Token::Report(s) => Token::Report(s[..split].to_string()),
            Token::Error(s) => Token::Error(s[..split].to_string()),
            _ => self.clone(),
        }
    }
}

struct TokenChunkIterator<I>
where
    I: Iterator<Item = Token>,
{
    iter: I,
    buffer: Option<Token>,
}

fn into_chunk(token: Token) -> Option<chunk::ActiveModel> {
    match token {
        Token::User(content) => Some(chunk::ActiveModel {
            kind: sea_orm::Set(ChunkKind::Text),
            content: sea_orm::Set(content),
            ..Default::default()
        }),
        Token::Message(content) => Some(chunk::ActiveModel {
            kind: sea_orm::Set(ChunkKind::Text),
            content: sea_orm::Set(content),
            ..Default::default()
        }),
        Token::Tool { name, args, id } => {
            let tool_call_info = ToolCallInfo {
                name,
                id,
                input: args,
                output: None,
            };
            let content = serde_json::to_string(&tool_call_info).unwrap();
            Some(chunk::ActiveModel {
                kind: sea_orm::Set(ChunkKind::ToolCall),
                content: sea_orm::Set(content),
                ..Default::default()
            })
        }
        Token::ToolResult(result) => Some(chunk::ActiveModel {
            kind: sea_orm::Set(ChunkKind::Error),
            content: sea_orm::Set("ToolResult not followed by tool call".to_string()),
            ..Default::default()
        }),
        Token::Reasoning(content) => Some(chunk::ActiveModel {
            kind: sea_orm::Set(ChunkKind::Reasoning),
            content: sea_orm::Set(content),
            ..Default::default()
        }),
        Token::Error(content) => Some(chunk::ActiveModel {
            kind: sea_orm::Set(ChunkKind::Error),
            content: sea_orm::Set(content),
            ..Default::default()
        }),
        _ => None,
    }
}

impl<I> Iterator for TokenChunkIterator<I>
where
    I: Iterator<Item = Token>,
{
    type Item = chunk::ActiveModel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_none() {
            return None;
        }
        let current = self.buffer.take().unwrap();

        let next = self.iter.next();
        if next.is_none() {
            return into_chunk(current).map(Some).unwrap_or_else(|| self.next());
        }
        let next = next.unwrap();

        let (merged_token, remaining) = current.merge(next);

        if remaining.is_some() {
            self.buffer = remaining;
            return into_chunk(merged_token)
                .map(Some)
                .unwrap_or_else(|| self.next());
        }

        self.buffer = Some(merged_token);
        self.next()
    }
}

impl Token {
    pub fn into_chunks<I: Iterator<Item = Self>>(
        mut tokens: I,
    ) -> impl Iterator<Item = chunk::ActiveModel> {
        return TokenChunkIterator {
            buffer: tokens.next(),
            iter: tokens,
        };
    }
}

impl From<openrouter::StreamCompletionResp> for Token {
    fn from(resp: openrouter::StreamCompletionResp) -> Self {
        match resp {
            StreamCompletionResp::ReasoningToken(reasoning) => Token::Reasoning(reasoning),
            StreamCompletionResp::ResponseToken(content) => Token::Message(content),
            StreamCompletionResp::ToolCall { name, args, id } => Token::Tool { name, args, id },
            _ => Token::Empty,
        }
    }
}
