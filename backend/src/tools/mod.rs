use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context, Result};
use axum::response::{IntoResponse, Response};
use schemars::{JsonSchema, schema_for};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::openrouter;

pub trait Tool: Serialize + DeserializeOwned + Default + 'static {
    /// for LLM
    type Input: JsonSchema + DeserializeOwned;
    type Output: IntoResponse;

    // State for the UI
    type State: Serialize;

    const NAME: &str;
    const DESCRIPTION: &str;

    fn call(&mut self, input: Self::Input) -> Result<(Self::Output, Self::State)>;
}

pub trait UntypedTool {
    fn call(&mut self, input: Value) -> Result<(Response, Value)>;
    fn se(self) -> Result<Value>;
}

trait ToolConstructor {
    fn new(&self, v: Value) -> Result<Box<dyn UntypedTool>>;
    fn default(&self) -> Box<dyn UntypedTool>;
}

struct Eraser<T> {
    // allow send
    _marker: PhantomData<fn() -> T>,
}

impl<T> Eraser<T>
where
    T: Tool,
{
    fn new() -> Box<dyn ToolConstructor + Send + Sync> {
        Box::new(Self {
            _marker: PhantomData,
        })
    }
}

impl<T> ToolConstructor for Eraser<T>
where
    T: Tool,
{
    fn new(&self, v: Value) -> Result<Box<dyn UntypedTool>> {
        let tool: T = serde_json::from_value(v).context("Cannot deserialize tool state")?;
        Ok(Box::new(tool))
    }

    fn default(&self) -> Box<dyn UntypedTool> {
        Box::new(T::default())
    }
}

impl<T> UntypedTool for T
where
    T: Tool,
{
    fn call(&mut self, input: Value) -> Result<(Response, Value)> {
        Tool::call(self, serde_json::from_value(input)?)
            .map(|(resp, state)| {
                serde_json::to_value(state)
                    .context("Cannot se the value")
                    .map(|state| (resp.into_response(), state))
            })
            .flatten()
    }

    fn se(self) -> Result<Value> {
        serde_json::to_value(self).context("Cannot se tool")
    }
}

#[derive(Default)]
pub struct ToolStore {
    tools: HashMap<&'static str, ToolStoreInner>,
}

pub struct ToolStoreInner {
    constructor: Box<dyn ToolConstructor + Send + Sync>,
    description: &'static str,
    schema: Value,
}

impl ToolStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_tool<T: Tool>(&mut self) -> Result<()> {
        self.tools.insert(
            T::NAME,
            ToolStoreInner {
                constructor: Eraser::<T>::new(),
                description: T::DESCRIPTION,
                schema: serde_json::to_value(schema_for!(T::Input))
                    .context("Cannot generate schema for tool")?,
            },
        );
        Ok(())
    }

    pub fn list(&self) -> Vec<openrouter::Tool> {
        self.tools
            .iter()
            .map(|(name, tool)| openrouter::Tool {
                name: name.to_string(),
                description: tool.description.to_owned(),
                schema: tool.schema.clone(),
            })
            .collect()
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Box<dyn UntypedTool>> {
        self.tools
            .get(name.as_ref())
            .map(|x| x.constructor.default())
    }
}
