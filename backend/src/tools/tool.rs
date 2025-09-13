use anyhow::{Context, Result};
use futures_util::{FutureExt, future::BoxFuture};
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub trait Tool: Serialize + DeserializeOwned + Default + Send + 'static {
    type Input: JsonSchema + DeserializeOwned + Send;
    type Output: Serialize;

    const NAME: &str;
    const DESCRIPTION: &str;
    const PROMPT: &str;

    fn call(&mut self, input: Self::Input) -> impl Future<Output = Result<Self::Output>> + Send;
}

pub trait UntypedTool: Send {
    fn call<'a>(&'a mut self, input: &'a str) -> BoxFuture<'a, Result<Value>>;
    fn se(&self) -> Result<String>;
}

impl<T> UntypedTool for T
where
    T: Tool,
{
    fn call<'a>(&'a mut self, input: &'a str) -> BoxFuture<'a, Result<Value>> {
        async {
            Ok(Tool::call(self, serde_json::from_str(input)?)
                .await
                .map(|output| serde_json::to_value(output))??)
        }
        .boxed()
    }

    fn se(&self) -> Result<String> {
        serde_json::to_string(&self).context("Cannot se tool")
    }
}
