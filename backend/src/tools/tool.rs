use anyhow::{Context, Result};
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub trait Tool: Serialize + DeserializeOwned + Default + 'static {
    /// for LLM
    type Input: JsonSchema + DeserializeOwned;
    type Output: IntoResponse;

    // State for the UI
    type State: Serialize;

    const NAME: &str;
    const DESCRIPTION: &str;
    const PROMPT: &str;

    fn call(&mut self, input: Self::Input) -> Result<(Self::Output, Self::State)>;
}

pub trait UntypedTool {
    fn call(&mut self, input: Value) -> Result<(Response, Value)>;
    fn se(&self) -> Result<String>;
}

impl<T> UntypedTool for T
where
    T: Tool,
{
    fn call(&mut self, input: Value) -> Result<(Response, Value)> {
        Tool::call(self, serde_json::from_value(input)?).map(|(resp, state)| {
            serde_json::to_value(state)
                .context("Cannot se the value")
                .map(|state| (resp.into_response(), state))
        })?
    }

    fn se(&self) -> Result<String> {
        serde_json::to_string(&self).context("Cannot se tool")
    }
}
