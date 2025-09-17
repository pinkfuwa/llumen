use super::channel::Mergeable;
use entity::chunk;

#[derive(Debug, Clone)]
pub enum Token {
    User(String),
    Message(String),
    Tool { name: String, args: String },
    ToolResult(String),
    Reasoning(String),
    Empty,
    Plan(String),
    Step(String),
    Report(String),
    Error(String),
}

impl Mergeable for Token {
    fn merge(self, other: Self) -> (Self, Option<Self>) {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn split_end(&self, split: usize) -> Option<Self> {
        todo!()
    }
    fn split_start(&self, split: usize) -> Self {
        todo!()
    }
}

struct TokenIterator<I>
where
    I: Iterator<Item = Token>,
{
    iter: I,
    buffer: Option<Token>,
}

impl<I> Iterator for TokenIterator<I>
where
    I: Iterator<Item = Token>,
{
    type Item = chunk::ActiveModel;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Token {
    pub fn into_chunks<I: Iterator<Item = Self>>(
        tokens: I,
    ) -> impl Iterator<Item = chunk::ActiveModel> {
        return TokenIterator {
            iter: tokens,
            buffer: None,
        };
    }
}
