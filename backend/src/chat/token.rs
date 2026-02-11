use super::channel::Mergeable;

/// Removes incomplete UTF-8 characters from the end of a string.
/// This ensures all strings in the buffer have complete UTF-8 sequences.
fn sanitize_utf8(s: &mut String) {
    while !s.is_empty() {
        match std::str::from_utf8(s.as_bytes()) {
            Ok(_) => break,
            Err(e) => {
                // Truncate at the position where invalid UTF-8 was found
                let valid_up_to = e.valid_up_to();
                s.truncate(valid_up_to);
            }
        }
    }
}

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
    UrlCitation(Vec<protocol::UrlCitation>),
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
            (Token::Assistant(s1), Token::Assistant(mut s2)) => {
                sanitize_utf8(&mut s2);
                s1.push_str(&s2);
                None
            }
            (Token::Reasoning(s1), Token::Reasoning(mut s2)) => {
                sanitize_utf8(&mut s2);
                s1.push_str(&s2);
                None
            }
            (Token::DeepStepToken(s1), Token::DeepStepToken(mut s2)) => {
                sanitize_utf8(&mut s2);
                s1.push_str(&s2);
                None
            }
            (Token::DeepStepReasoning(s1), Token::DeepStepReasoning(mut s2)) => {
                sanitize_utf8(&mut s2);
                s1.push_str(&s2);
                None
            }
            (Token::DeepReport(s1), Token::DeepReport(mut s2)) => {
                sanitize_utf8(&mut s2);
                s1.push_str(&s2);
                None
            }
            (Token::DeepPlan(s1), Token::DeepPlan(mut s2)) => {
                sanitize_utf8(&mut s2);
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
            | Token::Image(_)
            | Token::UrlCitation(_) => 1,
        }
    }

    fn slice(&self, r: std::ops::Range<usize>) -> Option<Self> {
        match self {
            Token::Assistant(s) => s.get(r).map(|slice| Token::Assistant(slice.to_string())),
            Token::Reasoning(s) => s.get(r).map(|slice| Token::Reasoning(slice.to_string())),
            Token::DeepStepToken(s) => s
                .get(r)
                .map(|slice| Token::DeepStepToken(slice.to_string())),
            Token::DeepStepReasoning(s) => s
                .get(r)
                .map(|slice| Token::DeepStepReasoning(slice.to_string())),
            Token::DeepReport(s) => s.get(r).map(|slice| Token::DeepReport(slice.to_string())),
            Token::Error(s) => s.get(r).map(|slice| Token::Error(slice.to_string())),
            Token::DeepPlan(s) => s.get(r).map(|slice| Token::DeepPlan(slice.to_string())),
            x if r.start == 0 => Some(x.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_utf8_removes_incomplete_chars() {
        // "嗨！发" in UTF-8 is: E5 98 A8 EF BC 81 E5 8F 91
        // If we cut it in the middle of "发", we should remove the incomplete bytes
        let mut bytes = Vec::from("嗨！".as_bytes());
        bytes.extend_from_slice(&[0xE5, 0x8F]); // incomplete UTF-8
        let mut incomplete = String::from_utf8_lossy(&bytes).into_owned();

        sanitize_utf8(&mut incomplete);

        // The incomplete bytes should be removed, leaving valid UTF-8
        assert!(std::str::from_utf8(incomplete.as_bytes()).is_ok());
        assert!(incomplete.starts_with("嗨！"));
    }

    #[test]
    fn test_sanitize_utf8_keeps_valid_strings() {
        let mut valid = String::from("Hello 世界");
        sanitize_utf8(&mut valid);
        assert_eq!(valid, "Hello 世界");
    }

    #[test]
    fn test_merge_with_incomplete_utf8() {
        let mut bytes1 = Vec::from("嗨！".as_bytes());
        bytes1.extend_from_slice(&[0xE5, 0x8F]); // incomplete UTF-8
        let incomplete_str = String::from_utf8_lossy(&bytes1).into_owned();

        let mut token1 = Token::Assistant(incomplete_str);
        let token2 = Token::Assistant(String::from("world"));

        let rest = token1.merge(token2);

        assert!(rest.is_none());
        if let Token::Assistant(s) = token1 {
            // After sanitization and merge, should be valid UTF-8
            assert!(std::str::from_utf8(s.as_bytes()).is_ok());
            assert!(s.contains("world"));
        } else {
            panic!("Expected Assistant token");
        }
    }

    #[test]
    fn test_slice_with_multibyte_chars() {
        let token = Token::Assistant(String::from("嗨！看起来你发了一个简单的「hi」"));

        // Test valid byte boundaries
        let sliced = token.slice(0..15);
        if let Some(Token::Assistant(s)) = sliced {
            // Should be valid UTF-8
            assert!(std::str::from_utf8(s.as_bytes()).is_ok());
        }

        // Invalid byte boundary should return None instead of panicking
        let invalid_slice = token.slice(0..10);
        assert!(
            invalid_slice.is_none(),
            "Slicing at invalid UTF-8 boundary should return None"
        );
    }

    #[test]
    fn test_len_and_slice_consistency() {
        let token = Token::Assistant(String::from("Hello 世界"));
        let len = token.len();

        // Slicing from 0 to len should work
        let sliced = token.slice(0..len);
        assert!(sliced.is_some());

        if let Some(Token::Assistant(s)) = sliced {
            assert_eq!(s, "Hello 世界");
        }
    }

    #[test]
    fn test_merge_preserves_utf8_validity() {
        let mut token1 = Token::DeepStepToken(String::from("测试"));
        let token2 = Token::DeepStepToken(String::from("内容"));

        token1.merge(token2);

        if let Token::DeepStepToken(s) = token1 {
            assert_eq!(s, "测试内容");
            assert!(std::str::from_utf8(s.as_bytes()).is_ok());
        }
    }

    #[test]
    fn test_regression_bug_report() {
        // Regression test for the exact bug in the report:
        // "byte index 23 is not a char boundary; it is inside '发' (bytes 22..25)"
        let text = "嗨！看起来你发了一个简单的「hi」，可能是";

        // Create incomplete UTF-8 by truncating mid-character
        let mut bytes = Vec::from(text.as_bytes());
        // Truncate in the middle of a character
        bytes.truncate(23);
        let incomplete_str = String::from_utf8_lossy(&bytes).into_owned();

        let mut token1 = Token::Assistant(incomplete_str);
        let token2 = Token::Assistant(String::from(" more text"));

        // This should not panic
        token1.merge(token2);

        if let Token::Assistant(s) = &token1 {
            assert!(std::str::from_utf8(s.as_bytes()).is_ok());
        }

        // Slicing should also not panic
        let len = token1.len();
        let sliced = token1.slice(0..len);
        assert!(sliced.is_some());

        // Even invalid boundaries should not panic
        let invalid_slice = token1.slice(0..10);
        // May be None if boundary is invalid, but should never panic
        let _ = invalid_slice;
    }
}
