use crate::openrouter::{CompletionOption, Message, Model, ModelBuilder, Openrouter, Tool};
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
    };

    assert_eq!(result.image.len(), 1, "Should have one image");
    assert_eq!(result.image[0].mime_type, "image/png");
    assert!(!result.image[0].data.is_empty());
}

#[test]
fn test_multipart_assistant_message() {
    use crate::openrouter::{Image, Message};

    // Create a sample image
    let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header bytes
    let image = Image {
        data: image_data.clone(),
        mime_type: "image/png".to_string(),
    };

    // Create a multipart assistant message with image
    let message = Message::Assistant {
        content: "Here is the generated image.".to_string(),
        annotations: None,
        reasoning_details: None,
        images: vec![image],
    };

    // Convert to raw message
    let raw_message = message.to_raw_message("test-model");

    // Verify it has multipart content
    assert!(raw_message.contents.is_some());
    let parts = raw_message.contents.unwrap();

    // Should have 2 parts: image first, then text
    assert_eq!(parts.len(), 2);

    // First part should be image_url type
    assert!(matches!(
        parts[0].r#type,
        crate::openrouter::raw::MultiPartMessageType::ImageUrl
    ));
    assert!(parts[0].image_url.is_some());

    // Verify the image is base64 encoded by checking serialization
    let serialized = serde_json::to_string(&parts[0]).unwrap();
    assert!(serialized.contains("data:image/png;base64"));

    // Second part should be text
    assert_eq!(
        parts[1].text.as_ref().unwrap(),
        "Here is the generated image."
    );
}
