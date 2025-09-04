use anyhow::{Context, Result};
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub trait Tool: Serialize + DeserializeOwned {
    /// for LLM
    type Input: JsonSchema + DeserializeOwned;
    type Output: IntoResponse;

    // State for the UI
    type State: Serialize;

    const NAME: &str;
    const DESCRIPTION: &str;

    fn call(&mut self, input: Self::Input) -> Result<(Self::Output, Self::State)>;
}

trait EraseTool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn erase_call(&mut self, input: serde_json::Value) -> Result<(Response, serde_json::Value)>;
}

impl<T> EraseTool for T
where
    T: Tool,
{
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn description(&self) -> &'static str {
        T::DESCRIPTION
    }

    fn erase_call(&mut self, input: serde_json::Value) -> Result<(Response, serde_json::Value)> {
        self.call(serde_json::from_value(input)?)
            .map(|(resp, state)| {
                serde_json::to_value(state)
                    .context("Cannot ser the value")
                    .map(|state| (resp.into_response(), state))
            })
            .flatten()
    }
}
