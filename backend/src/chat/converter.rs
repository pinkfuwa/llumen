//! Boundary adapters for data mapping between layers.
//!
//! This module centralises every type conversion that crosses a layer boundary:
//! - **SSE**: `Token` → `SseResp` (streaming to frontend)
//! - **OpenRouter**: `protocol::MessageInner` → `openrouter::Message` (to LLM)
//! - **Database**: streaming `Token`s → `protocol::AssistantChunk`
//!   (persistence)
//!
//! Keeping all conversions here prevents coupling between layers and makes
//! the mapping logic easy to test in isolation.

use crate::openrouter::{self, StreamCompletionResp};
use crate::routes::chat::sse::*;
use ::entity::message;
use protocol::{AssistantChunk, MessageInner};

use super::token::Token;

// ---------------------------------------------------------------------------
// SSE Adapter: Token → SseResp
// ---------------------------------------------------------------------------

/// Converts an internal streaming [`Token`] into the SSE response type sent to
/// the frontend.  Returns `None` for tokens that should be suppressed (e.g.
/// `Token::Empty`).
pub fn token_to_sse(token: Token) -> Option<SseResp> {
    match token {
        Token::Assistant(content) => Some(SseResp::Token(content)),
        Token::Reasoning(content) => Some(SseResp::Reasoning(content)),
        Token::ToolCall { name, arg } => {
            Some(SseResp::ToolCall(SseRespToolCall { name, args: arg }))
        }
        Token::ToolResult(content) => Some(SseResp::ToolResult(SseRespToolResult { content })),
        Token::Complete {
            message_id,
            token,
            cost,
        } => Some(SseResp::Complete(SseRespMessageComplete {
            id: message_id,
            token_count: token,
            cost,
            version: message_id,
        })),
        Token::Title(title) => Some(SseResp::Title(title)),
        Token::Error(content) => Some(SseResp::Error(content)),
        Token::Start { id, user_msg_id } => Some(SseResp::Start(SseStart {
            id,
            user_msg_id,
            version: user_msg_id,
        })),
        Token::DeepPlan(content) => Some(SseResp::DeepPlan(content)),
        Token::DeepStepStart(step) => Some(SseResp::DeepStepStart(step)),
        Token::DeepStepReasoning(content) => Some(SseResp::DeepStepReasoning(content)),
        Token::DeepStepToolCall { name, arg } => Some(SseResp::DeepStepToolCall(SseRespToolCall {
            name,
            args: arg,
        })),
        Token::DeepStepToolResult(content) => {
            Some(SseResp::DeepStepToolResult(SseRespToolResult { content }))
        }
        Token::DeepStepToken(content) => Some(SseResp::DeepStepToken(content)),
        Token::DeepReport(content) => Some(SseResp::DeepReport(content)),
        Token::Image(file_id) => Some(SseResp::Image(file_id)),
        Token::UrlCitation(citations) => Some(SseResp::UrlCitation(citations)),
        Token::Empty => None,
    }
}

// ---------------------------------------------------------------------------
// OpenRouter Adapter: protocol → openrouter::Message
// ---------------------------------------------------------------------------

/// Converts a slice of database message models into the `openrouter::Message`
/// sequence expected by the LLM API.
///
/// Each `MessageInner::Assistant` may expand into *multiple* OpenRouter
/// messages (e.g. text + tool-call + tool-result) because of how the
/// protocol packs everything into `Vec<AssistantChunk>`.
pub fn history_to_openrouter(
    history: &[message::Model],
    blob: &crate::utils::blob::BlobDB,
) -> Vec<openrouter::Message> {
    let mut messages = Vec::new();

    for msg in history {
        match &msg.inner {
            MessageInner::User { text, files } => {
                if files.is_empty() {
                    messages.push(openrouter::Message::User(text.clone()));
                } else {
                    let or_files = files
                        .iter()
                        .filter_map(|f| {
                            let reader = blob.get(f.id)?;
                            Some(openrouter::File {
                                name: f.name.clone(),
                                data: reader.as_ref().to_vec(),
                            })
                        })
                        .collect();
                    messages.push(openrouter::Message::MultipartUser {
                        text: text.clone(),
                        files: or_files,
                    });
                }
            }
            MessageInner::Assistant(chunks) => {
                chunks_to_openrouter(chunks, &mut messages);
            }
        }
    }

    messages
}

fn chunks_to_openrouter(chunks: &[AssistantChunk], out: &mut Vec<openrouter::Message>) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut annotations: Option<serde_json::Value> = None;
    let mut reasoning_details: Option<serde_json::Value> = None;
    let mut images: Vec<openrouter::Image> = Vec::new();

    let flush = |text_parts: &mut Vec<String>,
                 images: &mut Vec<openrouter::Image>,
                 out: &mut Vec<openrouter::Message>| {
        if text_parts.is_empty() && images.is_empty() {
            return;
        }
        out.push(openrouter::Message::Assistant {
            content: text_parts.join(""),
            annotations: None,
            reasoning_details: None,
            images: std::mem::take(images),
        });
        text_parts.clear();
    };

    for chunk in chunks {
        match chunk {
            AssistantChunk::Text(t) => text_parts.push(t.clone()),
            AssistantChunk::Reasoning(_) => {}
            AssistantChunk::ReasoningDetail(rd) => {
                reasoning_details = Some(rd.clone());
            }
            AssistantChunk::Annotation(a) => {
                annotations = Some(a.clone());
            }
            AssistantChunk::ToolCall { id, name, arg } => {
                flush(&mut text_parts, &mut images, out);
                out.push(openrouter::Message::ToolCall(openrouter::MessageToolCall {
                    id: id.clone(),
                    name: name.clone(),
                    arguments: arg.clone(),
                }));
            }
            AssistantChunk::ToolResult { id, response } => {
                out.push(openrouter::Message::ToolResult(
                    openrouter::MessageToolResult {
                        id: id.clone(),
                        content: response.clone(),
                    },
                ));
            }
            AssistantChunk::Image(_file_id) => {
                // Image reconstruction from blob not supported in history
                // replay
            }
            AssistantChunk::UrlCitation(_) | AssistantChunk::Error(_) => {}
            AssistantChunk::DeepAgent(_) => {}
        }
    }

    // Final flush with annotations and reasoning_details
    if !text_parts.is_empty() || !images.is_empty() {
        out.push(openrouter::Message::Assistant {
            content: text_parts.join(""),
            annotations,
            reasoning_details,
            images,
        });
    }
}

// ---------------------------------------------------------------------------
// OpenRouter Stream → Token (for strategies)
// ---------------------------------------------------------------------------

/// Maps an OpenRouter stream chunk into a normal-mode `Token`.
pub fn openrouter_to_buffer_token(resp: StreamCompletionResp) -> Token {
    match resp {
        StreamCompletionResp::ResponseToken(delta) => Token::Assistant(delta),
        StreamCompletionResp::ReasoningToken(delta) => Token::Reasoning(delta),
        StreamCompletionResp::ToolToken { name, args, .. } => Token::ToolCall { name, arg: args },
        StreamCompletionResp::Usage { .. } => Token::Empty,
    }
}

/// Maps an OpenRouter stream chunk into a deep-step `Token`.
pub fn openrouter_to_buffer_token_deep_step(resp: StreamCompletionResp) -> Token {
    match resp {
        StreamCompletionResp::ResponseToken(delta) => Token::DeepStepToken(delta),
        StreamCompletionResp::ReasoningToken(delta) => Token::DeepStepReasoning(delta),
        StreamCompletionResp::ToolToken { name, args, .. } => {
            Token::DeepStepToolCall { name, arg: args }
        }
        StreamCompletionResp::Usage { .. } => Token::Empty,
    }
}

/// Maps an OpenRouter stream chunk into a deep-report `Token`.
pub fn openrouter_to_buffer_token_deep_report(resp: StreamCompletionResp) -> Token {
    match resp {
        StreamCompletionResp::ResponseToken(delta) => Token::DeepReport(delta),
        StreamCompletionResp::ReasoningToken(delta) => Token::DeepReport(delta),
        StreamCompletionResp::ToolToken { .. } | StreamCompletionResp::Usage { .. } => Token::Empty,
    }
}

// ---------------------------------------------------------------------------
// OpenRouter StreamResult → protocol::AssistantChunk (persistence)
// ---------------------------------------------------------------------------

/// Converts accumulated stream response items into storable assistant chunks.
/// Merges consecutive reasoning tokens into a single chunk (like Token::merge).
pub fn openrouter_stream_to_assitant_chunk(
    responses: &[StreamCompletionResp],
) -> Vec<AssistantChunk> {
    let mut chunks = Vec::new();
    let mut text = String::new();
    let mut reasoning = String::new();

    for resp in responses {
        match resp {
            StreamCompletionResp::ResponseToken(delta) => {
                // Flush reasoning if we're switching to text
                if !reasoning.is_empty() {
                    chunks.push(AssistantChunk::Reasoning(std::mem::take(&mut reasoning)));
                }
                text.push_str(delta);
            }
            StreamCompletionResp::ReasoningToken(delta) => {
                // Flush text if we're switching to reasoning
                if !text.is_empty() {
                    chunks.push(AssistantChunk::Text(std::mem::take(&mut text)));
                }
                reasoning.push_str(delta);
            }
            StreamCompletionResp::ToolToken { .. } | StreamCompletionResp::Usage { .. } => {}
        }
    }

    // Flush remaining
    if !text.is_empty() {
        chunks.push(AssistantChunk::Text(text));
    }
    if !reasoning.is_empty() {
        chunks.push(AssistantChunk::Reasoning(reasoning));
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_to_sse_maps_all_variants() {
        assert!(token_to_sse(Token::Empty).is_none());

        let sse = token_to_sse(Token::Assistant("hi".into())).unwrap();
        assert!(matches!(sse, SseResp::Token(s) if s == "hi"));

        let sse = token_to_sse(Token::ToolCall {
            name: "web_search".into(),
            arg: "{}".into(),
        })
        .unwrap();
        assert!(matches!(sse, SseResp::ToolCall(tc) if tc.name == "web_search"));
    }

    #[test]
    fn stream_token_to_buffer_token() {
        let token = openrouter_to_buffer_token(StreamCompletionResp::ResponseToken("hello".into()));
        assert!(matches!(token, Token::Assistant(s) if s == "hello"));

        let token =
            openrouter_to_buffer_token(StreamCompletionResp::ReasoningToken("think".into()));
        assert!(matches!(token, Token::Reasoning(s) if s == "think"));
    }

    #[test]
    fn openrouter_stream_merges_reasoning_chunks() {
        // Test that consecutive reasoning tokens are merged into a single chunk
        let responses = vec![
            StreamCompletionResp::ReasoningToken("<thinking>".into()),
            StreamCompletionResp::ReasoningToken("The user".into()),
            StreamCompletionResp::ReasoningToken(" wants".into()),
            StreamCompletionResp::ReasoningToken("</thinking>".into()),
            StreamCompletionResp::ResponseToken("Hello".into()),
            StreamCompletionResp::ResponseToken(" world".into()),
        ];

        let chunks = openrouter_stream_to_assitant_chunk(&responses);

        assert_eq!(chunks.len(), 2);
        assert!(
            matches!(&chunks[0], AssistantChunk::Reasoning(s) if s == "<thinking>The user wants</thinking>")
        );
        assert!(matches!(&chunks[1], AssistantChunk::Text(s) if s == "Hello world"));
    }

    #[test]
    fn openrouter_stream_interleaves_text_and_reasoning() {
        // Test text -> reasoning -> text transitions
        let responses = vec![
            StreamCompletionResp::ResponseToken("Before".into()),
            StreamCompletionResp::ReasoningToken("<think>".into()),
            StreamCompletionResp::ReasoningToken("thinking".into()),
            StreamCompletionResp::ReasoningToken("</think>".into()),
            StreamCompletionResp::ResponseToken("After".into()),
        ];

        let chunks = openrouter_stream_to_assitant_chunk(&responses);

        assert_eq!(chunks.len(), 3);
        assert!(matches!(&chunks[0], AssistantChunk::Text(s) if s == "Before"));
        assert!(
            matches!(&chunks[1], AssistantChunk::Reasoning(s) if s == "<think>thinking</think>")
        );
        assert!(matches!(&chunks[2], AssistantChunk::Text(s) if s == "After"));
    }

    #[test]
    fn chunks_to_openrouter_preserves_annotations_and_reasoning() {
        use serde_json::json;

        // Test that annotations and reasoning_details are attached to the final message
        let annotation = json!({"type": "ocr", "data": "cached_result"});
        let reasoning = json!({"model_id": "gpt-4", "data": "thinking"});

        let chunks = vec![
            AssistantChunk::Text("Hello".into()),
            AssistantChunk::Annotation(annotation.clone()),
            AssistantChunk::ReasoningDetail(reasoning.clone()),
            AssistantChunk::Text(" world".into()),
        ];

        let mut messages = Vec::new();
        chunks_to_openrouter(&chunks, &mut messages);

        assert_eq!(messages.len(), 1);
        if let openrouter::Message::Assistant {
            content,
            annotations,
            reasoning_details,
            ..
        } = &messages[0]
        {
            assert_eq!(content, "Hello world");
            assert_eq!(annotations, &Some(annotation));
            assert_eq!(reasoning_details, &Some(reasoning));
        } else {
            panic!("Expected Assistant message");
        }
    }

    #[test]
    fn chunks_to_openrouter_preserves_metadata_with_tool_calls() {
        use serde_json::json;

        // Test that annotations/reasoning persist through tool calls
        let annotation = json!({"type": "ocr"});
        let reasoning = json!({"model_id": "gpt-4", "data": "thinking"});

        let chunks = vec![
            AssistantChunk::Text("First".into()),
            AssistantChunk::Annotation(annotation.clone()),
            AssistantChunk::ReasoningDetail(reasoning.clone()),
            AssistantChunk::ToolCall {
                id: "call_1".into(),
                name: "search".into(),
                arg: "{}".into(),
            },
            AssistantChunk::ToolResult {
                id: "call_1".into(),
                response: "result".into(),
            },
            AssistantChunk::Text("Second".into()),
        ];

        let mut messages = Vec::new();
        chunks_to_openrouter(&chunks, &mut messages);

        // Should have: Assistant(First) -> ToolCall -> ToolResult -> Assistant(Second
        // with metadata)
        assert_eq!(messages.len(), 4);

        // First assistant message should NOT have metadata (flushed before tool call)
        if let openrouter::Message::Assistant {
            annotations,
            reasoning_details,
            ..
        } = &messages[0]
        {
            assert_eq!(annotations, &None);
            assert_eq!(reasoning_details, &None);
        }

        // Last assistant message should have metadata
        if let openrouter::Message::Assistant {
            content,
            annotations,
            reasoning_details,
            ..
        } = &messages[3]
        {
            assert_eq!(content, "Second");
            assert_eq!(annotations, &Some(annotation));
            assert_eq!(reasoning_details, &Some(reasoning));
        } else {
            panic!("Expected Assistant message at end");
        }
    }

    #[test]
    fn chunks_roundtrip_preserves_metadata() {
        use serde_json::json;

        // Test full roundtrip: chunks -> messages -> (would save to DB) -> load chunks
        // -> messages
        let annotation = json!({"type": "ocr", "cached": true});
        let reasoning =
            json!({"model_id": "gpt-4", "data": [{"type": "reasoning", "content": "thinking"}]});

        // Simulate what gets stored in DB after a completion
        let stored_chunks = vec![
            AssistantChunk::Text("Answer: ".into()),
            AssistantChunk::Reasoning("Let me think...".into()),
            AssistantChunk::Text("42".into()),
            AssistantChunk::Annotation(annotation.clone()),
            AssistantChunk::ReasoningDetail(reasoning.clone()),
        ];

        // Convert to messages (what gets sent to OpenRouter)
        let mut messages = Vec::new();
        chunks_to_openrouter(&stored_chunks, &mut messages);

        // Verify metadata is preserved
        assert_eq!(messages.len(), 1);
        if let openrouter::Message::Assistant {
            content,
            annotations: msg_ann,
            reasoning_details: msg_rd,
            ..
        } = &messages[0]
        {
            assert_eq!(content, "Answer: 42");
            assert_eq!(msg_ann, &Some(annotation));
            assert_eq!(msg_rd, &Some(reasoning));
        } else {
            panic!("Expected Assistant message");
        }
    }
}
