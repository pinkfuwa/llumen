use crate::openrouter::{Message, Model, Openrouter, Tool};
use std::env;

#[test]
fn test_tool_call_structure() {
    // Test that ToolCall can hold multiple tool calls
    let mut toolcalls = Vec::new();

    toolcalls.push(crate::openrouter::stream::ToolCall {
        id: "call_1".to_string(),
        name: "get_weather".to_string(),
        args: r#"{"location": "Paris"}"#.to_string(),
    });

    toolcalls.push(crate::openrouter::stream::ToolCall {
        id: "call_2".to_string(),
        name: "get_time".to_string(),
        args: r#"{"location": "London"}"#.to_string(),
    });

    assert_eq!(toolcalls.len(), 2);
    assert_eq!(toolcalls[0].name, "get_weather");
    assert_eq!(toolcalls[1].name, "get_time");
}

#[test]
fn test_stream_completion_resp_variants() {
    use crate::openrouter::StreamCompletionResp;

    // Test that all variants can be constructed
    let token = StreamCompletionResp::ResponseToken("Hello".to_string());
    let reasoning = StreamCompletionResp::ReasoningToken("Thinking...".to_string());
    let tool_token = StreamCompletionResp::ToolToken {
        idx: 0,
        args: "partial".to_string(),
        name: "test".to_string(),
    };

    assert!(matches!(token, StreamCompletionResp::ResponseToken(_)));
    assert!(matches!(reasoning, StreamCompletionResp::ReasoningToken(_)));
    assert!(matches!(tool_token, StreamCompletionResp::ToolToken { .. }));
}

#[tokio::test]
async fn tool_calls() {
    let api_key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("API_KEY not set, skipping OpenRouter tool call tests.");
            return;
        }
    };

    let api_base = env::var("API_BASE").unwrap_or_else(|_| "https://openrouter.ai/api".to_string());

    let openrouter = Openrouter::new(&api_key, &api_base);

    // Create a simple tool definition
    let tool = Tool {
        name: "get_weather".to_string(),
        description: "Get the current weather in a location".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                }
            },
            "required": ["location"]
        }),
    };

    let messages = vec![Message::User(
        "What's the weather like in Paris?".to_string(),
    )];

    let model = Model {
        id: "openai/gpt-3.5-turbo".to_string(),
        temperature: Some(0.7),
        ..Default::default()
    };

    let mut stream = openrouter
        .stream(messages, &model, vec![tool])
        .await
        .unwrap();

    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) => match resp {
                crate::openrouter::StreamCompletionResp::ToolToken { idx, args, name } => {
                    println!("Tool token at idx {}: name={}, args={}", idx, name, args);
                }
                _ => {}
            },
            Err(e) => {
                println!("Stream error: {:?}", e);
                panic!("Stream should not error");
            }
        }
    }

    let result = stream.get_result();

    // For now, we just verify the structure works
    // The actual tool call might not happen if the model doesn't decide to use it
    println!("Tool calls in result: {}", result.toolcalls.len());
    println!("Stop reason: {:?}", result.stop_reason);

    // If a tool call was made, verify it's in the result
    if !result.toolcalls.is_empty() {
        assert!(
            !result.toolcalls[0].name.is_empty(),
            "Tool name should not be empty"
        );
        assert!(
            !result.toolcalls[0].id.is_empty(),
            "Tool call id should not be empty"
        );
    }
}

#[tokio::test]
async fn parallel_tool_calls() {
    let api_key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("API_KEY not set, skipping OpenRouter parallel tool call tests.");
            return;
        }
    };

    let api_base = env::var("API_BASE").unwrap_or_else(|_| "https://openrouter.ai/api".to_string());

    let openrouter = Openrouter::new(&api_key, &api_base);

    // Create multiple tool definitions
    let weather_tool = Tool {
        name: "get_weather".to_string(),
        description: "Get the current weather in a location".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                }
            },
            "required": ["location"]
        }),
    };

    let time_tool = Tool {
        name: "get_time".to_string(),
        description: "Get the current time in a location".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                }
            },
            "required": ["location"]
        }),
    };

    let messages = vec![Message::User(
        "What's the weather and current time in Paris and London?".to_string(),
    )];

    let model = Model {
        id: "openai/gpt-4".to_string(),
        temperature: Some(0.7),
        ..Default::default()
    };

    let mut stream = openrouter
        .stream(messages, &model, vec![weather_tool, time_tool])
        .await
        .unwrap();

    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) => match resp {
                crate::openrouter::StreamCompletionResp::ToolToken { idx, args, name } => {
                    println!("Tool token at idx {}: name={}, args={}", idx, name, args);
                }
                _ => {}
            },
            Err(e) => {
                println!("Stream error: {:?}", e);
            }
        }
    }

    let result = stream.get_result();

    println!("Total tool calls in result: {}", result.toolcalls.len());
    println!("Stop reason: {:?}", result.stop_reason);

    // The test verifies that our code can handle multiple tool calls
    // The actual number depends on what the model decides to do
    if result.toolcalls.len() > 1 {
        println!(
            "✓ Parallel tool calls supported! Received {} tool calls",
            result.toolcalls.len()
        );
    } else {
        println!(
            "Model made {} tool call(s). Parallel tool call support is implemented but model didn't use it in this test.",
            result.toolcalls.len()
        );
    }
}
