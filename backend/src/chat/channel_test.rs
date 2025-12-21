use super::channel::*;
use super::token::Token;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_first_token_not_missing() {
    // Create a channel context
    let ctx = std::sync::Arc::new(Context::<Token>::new());
    
    // Create a publisher
    let mut publisher = ctx.clone().publish(1).expect("Failed to create publisher");
    
    // Subscribe before publishing
    let mut stream = ctx.clone().subscribe(1, None);
    
    // Publish tokens
    publisher.publish(Token::Start {
        id: 1,
        user_msg_id: 0,
    });
    
    publisher.publish(Token::Assistant("Hello".to_string()));
    publisher.publish(Token::Assistant(" world".to_string()));
    
    // Drop publisher to signal end
    drop(publisher);
    
    // Collect all tokens
    let mut tokens = Vec::new();
    while let Some(token) = stream.next().await {
        tokens.push(token);
    }
    
    // Verify we got all tokens
    assert_eq!(tokens.len(), 2, "Should have 2 tokens (Start + merged Assistant)");
    
    // First token should be Start
    assert!(matches!(tokens[0], Token::Start { .. }), "First token should be Start");
    
    // Second token should be merged Assistant
    if let Token::Assistant(text) = &tokens[1] {
        assert_eq!(text, "Hello world", "Assistant token should be 'Hello world', got '{}'", text);
    } else {
        panic!("Second token should be Assistant, got {:?}", tokens[1]);
    }
}

#[tokio::test]
async fn test_subscribe_after_publish() {
    // Test subscribing after some tokens have already been published
    let ctx = std::sync::Arc::new(Context::<Token>::new());
    
    // Create publisher and publish some tokens
    let mut publisher = ctx.clone().publish(2).expect("Failed to create publisher");
    
    publisher.publish(Token::Start { id: 2, user_msg_id: 0 });
    publisher.publish(Token::Assistant("First".to_string()));
    
    // Now subscribe (after tokens published)
    let mut stream = ctx.clone().subscribe(2, None);
    
    // Publish more tokens
    publisher.publish(Token::Assistant(" Second".to_string()));
    
    drop(publisher);
    
    // Collect tokens
    let mut tokens = Vec::new();
    while let Some(token) = stream.next().await {
        tokens.push(token);
    }
    
    // Should see all tokens from the beginning
    assert_eq!(tokens.len(), 2, "Should have 2 tokens");
    assert!(matches!(tokens[0], Token::Start { .. }), "First should be Start");
    
    if let Token::Assistant(text) = &tokens[1] {
        assert_eq!(text, "First Second", "Should have merged text, got '{}'", text);
    } else {
        panic!("Second token should be Assistant");
    }
}

#[tokio::test]
async fn test_empty_tokens_filtered() {
    let ctx = std::sync::Arc::new(Context::<Token>::new());
    let mut publisher = ctx.clone().publish(3).expect("Failed to create publisher");
    let mut stream = ctx.clone().subscribe(3, None);
    
    // Publish empty token
    publisher.publish(Token::Assistant("".to_string()));
    // Publish non-empty token
    publisher.publish(Token::Assistant("Hello".to_string()));
    
    drop(publisher);
    
    let mut tokens = Vec::new();
    while let Some(token) = stream.next().await {
        tokens.push(token);
    }
    
    // Empty tokens should be filtered out
    assert_eq!(tokens.len(), 1, "Should have 1 token (empty filtered)");
    
    if let Token::Assistant(text) = &tokens[0] {
        assert_eq!(text, "Hello", "Should be 'Hello', got '{}'", text);
    } else {
        panic!("Token should be Assistant");
    }
}
