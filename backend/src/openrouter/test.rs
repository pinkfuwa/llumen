use crate::openrouter::{Message, Model, Openrouter};
use std::env;
use tokio::try_join;

#[tokio::test]
async fn tool_calls() {
    let api_key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("API_KEY not set, skipping OpenRouter tool call tests.");
            return;
        }
    };

    todo!()
}

#[tokio::test]
async fn parrallel_tool_calls() {
    let api_key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("API_KEY not set, skipping OpenRouter tool call tests.");
            return;
        }
    };

    todo!()
}
