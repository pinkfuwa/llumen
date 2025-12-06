use super::channel::Mergeable;

#[derive(Debug, Clone)]
pub enum Token {
    Assistant(String),
    ToolCall {
        name: String,
        arg: String,
    },
    // result json of tool, tools are called sequentially
    // For example, ToolCall(1)->ToolCall(2)->ToolCall(3), then first ToolResult are for first call
    ToolResult(String),
    Reasoning(String),
    Empty,
    DeepPlan(String),
    DeepStepStart(i32),
    DeepStepReasoning(String),
    DeepStepToolCall {
        name: String,
        arg: String,
    },
    DeepStepToolResult(String),
    DeepStepToken(String),
    DeepReport(String),
    Error(String),
    Image(i32),
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
            (Token::DeepStepToken(s1), Token::DeepStepToken(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::DeepStepReasoning(s1), Token::DeepStepReasoning(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::DeepReport(s1), Token::DeepReport(s2)) => {
                s1.push_str(&s2);
                None
            }
            (Token::DeepPlan(s1), Token::DeepPlan(s2)) => {
                s1.push_str(&s2);
                None
            }
            (_, other) => Some(other),
        }
    }

    fn len(&self) -> usize {
        match self {
            Token::Assistant(s)
            | Token::Reasoning(s)
            | Token::DeepStepToken(s)
            | Token::DeepStepReasoning(s)
            | Token::DeepReport(s)
            | Token::Error(s)
            | Token::DeepPlan(s) => s.len(),
            Token::ToolResult(_)
            | Token::Empty
            | Token::DeepStepStart(_)
            | Token::DeepStepToolResult(_)
            | Token::Complete { .. }
            | Token::Title(_)
            | Token::Start { .. }
            | Token::ToolCall { .. }
            | Token::DeepStepToolCall { .. }
            | Token::Image(_) => 1,
        }
    }

    fn slice(&self, r: std::ops::Range<usize>) -> Option<Self> {
        match self {
            Token::Assistant(s) => Some(Token::Assistant(s[r].to_string())),
            Token::Reasoning(s) => Some(Token::Reasoning(s[r].to_string())),
            Token::DeepStepToken(s) => Some(Token::DeepStepToken(s[r].to_string())),
            Token::DeepStepReasoning(s) => Some(Token::DeepStepReasoning(s[r].to_string())),
            Token::DeepReport(s) => Some(Token::DeepReport(s[r].to_string())),
            Token::Error(s) => Some(Token::Error(s[r].to_string())),
            Token::DeepPlan(s) => Some(Token::DeepPlan(s[r].to_string())),
            x if r.start == 0 => Some(x.clone()),
            _ => None,
        }
    }
}
