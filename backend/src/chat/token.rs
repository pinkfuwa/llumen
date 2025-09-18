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
    Assitant(String),
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
    fn merge(&mut self, other: Self) -> Option<Self> {
        match (self, other) {
            (Token::Assitant(s1), Token::Assitant(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::Reasoning(s1), Token::Reasoning(s2)) => {
                s1.push_str(&s2);
                None
            }
            (_, other) => Some(other),
        }
    }

    fn len(&self) -> usize {
        match self {
            Token::User(s)
            | Token::Assitant(s)
            | Token::Reasoning(s)
            | Token::Plan(s)
            | Token::Step(s)
            | Token::Report(s)
            | Token::Error(s) => s.len(),
            Token::Tool { .. }
            | Token::ToolResult { .. }
            | Token::Empty
            | Token::Complete { .. } => 1,
        }
    }

    fn split_end(&self, split: usize) -> Option<Self> {
        match self {
            Token::User(s) => Some(Token::User(s[split..].to_string())),
            Token::Assitant(s) => Some(Token::Assitant(s[split..].to_string())),
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
            Token::Assitant(s) => Token::Assitant(s[..split].to_string()),
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
        Token::Assitant(content) => Some(chunk::ActiveModel {
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
        let mut current = self.buffer.take().unwrap();

        let next = self.iter.next();
        if next.is_none() {
            return into_chunk(current).map(Some).unwrap_or_else(|| self.next());
        }
        let next = next.unwrap();

        let remaining = current.merge(next);

        // FIXME: special case => merge tool result with tool call

        if remaining.is_some() {
            self.buffer = remaining;
            return into_chunk(current).map(Some).unwrap_or_else(|| self.next());
        }

        self.buffer = Some(current);
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
            StreamCompletionResp::ResponseToken(content) => Token::Assitant(content),
            StreamCompletionResp::ToolCall { name, args, id } => Token::Tool { name, args, id },
            _ => Token::Empty,
        }
    }
}
