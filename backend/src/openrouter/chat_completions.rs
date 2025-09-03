use anyhow::{Context, Result, anyhow};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};

pub struct Completion(EventSource);

impl Completion {
    pub async fn request(
        messages: Vec<Message>,
        model: String,
        api_key: &str,
    ) -> Result<Completion> {
        let req = CompletionReq {
            messages,
            stream: true,
            model,
        };

        let builder = Client::new()
            .post(" https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&req);

        let es = EventSource::new(builder)?;

        Ok(Self(es))
    }

    pub fn close(&mut self) {
        self.0.close();
    }

    pub async fn next(&mut self) -> Option<Result<CompletionResp>> {
        loop {
            match self.0.next().await {
                Some(x) => match x {
                    Ok(e) => match e {
                        Event::Open => continue,
                        Event::Message(e) => {
                            if &e.data == "[DONE]" {
                                return None;
                            } else {
                                return Some(
                                    serde_json::from_str::<CompletionResp>(&e.data)
                                        .context("Parse error"),
                                );
                            }
                        }
                    },
                    Err(e) => match e {
                        reqwest_eventsource::Error::StreamEnded => return None,
                        e => return Some(Err(anyhow!("{e}"))),
                    },
                },
                None => return None,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CompletionReq {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResp {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub object: String,
    pub created: i64,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i64,
    pub delta: Delta,
    pub finish_reason: Option<serde_json::Value>,
    pub native_finish_reason: Option<serde_json::Value>,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    pub role: Role,
    pub content: String,
}
