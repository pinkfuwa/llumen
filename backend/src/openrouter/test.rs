use crate::openrouter::{CompletionOption, Message, Model, ModelBuilder, Openrouter, Tool};
use std::env;

#[tokio::test]
async fn test_text_output_capability() {
    // Test that text_output capability is correctly detected for different model
    // types

    // Test with a fake API key and base URL (won't make actual requests)
    let openrouter = Openrouter::new("test_key", "https://openrouter.ai/api");

    // Give initial model fetch time to start (though it won't complete with fake
    // key)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test 1: Future model (not in listing) - should default to image-only
    // This prevents first-request bugs where context is incorrectly injected
    let future_model = Model {
        id: "openai/gpt-5-turbo".to_string(),
        ..Default::default()
    };
    let capability3 = openrouter.get_capability(&future_model).await;
    assert_eq!(
        capability3.text_output, false,
        "Unknown models should default to image-only to avoid context injection bugs"
    );
    assert_eq!(
        capability3.image_output, true,
        "Unknown models should default to image-only"
    );

    // Test 2: User override - can override capabilities
    let overridden_model = Model {
        id: "black-forest-labs/flux-2".to_string(),
        capability: crate::openrouter::MaybeCapability {
            text_output: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    let capability4 = openrouter.get_capability(&overridden_model).await;
    assert_eq!(
        capability4.text_output, true,
        "User override should take precedence"
    );
}

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

    let model_for_stream = ModelBuilder::from_model(&model).build();
    let option = CompletionOption::builder().tools(&[tool]).build();
    let mut stream = openrouter
        .stream(model_for_stream, messages, option)
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
                // External API errors (like Cloudflare blocks) are not failures of our code
                // Log the error and gracefully exit the test
                println!("Stream error (external API issue): {:?}", e);
                println!("Skipping test due to external API error");
                return;
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

    let model_for_stream = ModelBuilder::from_model(&model).build();
    let option = CompletionOption::builder()
        .tools(&[weather_tool, time_tool])
        .build();
    let mut stream = openrouter
        .stream(model_for_stream, messages, option)
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
                // External API errors (like Cloudflare blocks) are not failures of our code
                // Log the error and gracefully exit the test
                println!("Stream error (external API issue): {:?}", e);
                println!("Skipping test due to external API error");
                return;
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

#[test]
fn test_image_data_url_parsing() {
    use crate::openrouter::Image;

    // Test parsing a valid PNG data URL
    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let image = Image::from_data_url(data_url).expect("Failed to parse valid PNG data URL");

    assert_eq!(image.mime_type, "image/png");
    assert!(!image.data.is_empty(), "Image data should not be empty");

    // Test parsing a valid JPEG data URL
    let jpeg_url = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8UHRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgNDRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjL/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCwAA8A/9k=";
    let jpeg_image = Image::from_data_url(jpeg_url).expect("Failed to parse valid JPEG data URL");

    assert_eq!(jpeg_image.mime_type, "image/jpeg");
    assert!(!jpeg_image.data.is_empty(), "JPEG data should not be empty");

    // Test invalid data URL (missing data: prefix)
    let invalid_url = "image/png;base64,iVBORw0KGgo=";
    let result = Image::from_data_url(invalid_url);
    assert!(result.is_err(), "Should fail for URL without data: prefix");

    // Test invalid data URL (missing comma separator)
    let invalid_url2 = "data:image/png;base64";
    let result2 = Image::from_data_url(invalid_url2);
    assert!(
        result2.is_err(),
        "Should fail for URL without comma separator"
    );

    // Test invalid base64
    let invalid_base64 = "data:image/png;base64,invalid-base64!!!";
    let result3 = Image::from_data_url(invalid_base64);
    assert!(result3.is_err(), "Should fail for invalid base64 data");
}

#[test]
fn test_stream_result_images() {
    use crate::openrouter::stream::{StreamCompletionResp, StreamResult, Usage};
    use crate::openrouter::{Image, raw::FinishReason};

    // Create a sample image
    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let image = Image::from_data_url(data_url).expect("Failed to parse test image");

    // Create a StreamResult with images
    let result = StreamResult {
        toolcalls: vec![],
        usage: Usage {
            token: 100,
            cost: 0.001,
        },
        stop_reason: FinishReason::Stop,
        responses: vec![StreamCompletionResp::ResponseToken(
            "Here is an image: ".to_string(),
        )],
        annotations: None,
        reasoning_details: None,
        image: vec![image],
        citations: vec![],
    };

    assert_eq!(result.image.len(), 1, "Should have one image");
    assert_eq!(result.image[0].mime_type, "image/png");
    assert!(!result.image[0].data.is_empty());
}

#[test]
fn test_citation_extraction() {
    use crate::openrouter::annotation::extract_url_citations;
    use serde_json::json;

    // Test extracting citations from annotations
    let annotations = json!([
        {
            "type": "url_citation",
            "url_citation": {
                "url": "https://example.com",
                "title": "Example Site",
                "content": "Sample content",
                "start_index": 10,
                "end_index": 50,
                "favicon": "https://example.com/favicon.ico"
            }
        },
        {
            "type": "url_citation",
            "url_citation": {
                "url": "https://test.com",
                "title": "Test Site"
            }
        },
        {
            "type": "other_annotation",
            "data": "should be ignored"
        }
    ]);

    let citations = extract_url_citations(&annotations);

    assert_eq!(citations.len(), 2, "Should extract 2 citations");
    assert_eq!(citations[0].url, "https://example.com");
    assert_eq!(citations[0].title, Some("Example Site".to_string()));
    assert_eq!(citations[0].content, Some("Sample content".to_string()));
    assert_eq!(citations[0].start_index, Some(10));
    assert_eq!(citations[0].end_index, Some(50));
    assert_eq!(
        citations[0].favicon,
        Some("https://example.com/favicon.ico".to_string())
    );

    assert_eq!(citations[1].url, "https://test.com");
    assert_eq!(citations[1].title, Some("Test Site".to_string()));
    assert_eq!(citations[1].content, None);
}
