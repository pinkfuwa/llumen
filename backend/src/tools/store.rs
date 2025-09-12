use std::{collections::HashMap, marker::PhantomData};

use anyhow::{Context, Result};
use entity::tool;
use schemars::schema_for;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::OnConflict;
use sea_orm::{DbConn, EntityTrait};
use serde_json::Value;

use crate::{
    openrouter,
    tools::{Tool, ToolSet, UntypedTool},
};

pub struct ToolStore {
    tools: HashMap<&'static str, ToolStoreInner>,
    conn: DbConn,
}

pub struct ToolStoreInner {
    constructor: Box<dyn ToolConstructor + Send + Sync>,
    description: &'static str,
    prompt: &'static str,
    schema: Value,
}

pub struct ToolBox {
    pub tools: HashMap<&'static str, Box<dyn UntypedTool>>,
    chat_id: i32,
}

impl ToolStore {
    pub fn new(conn: DbConn) -> Self {
        Self {
            tools: Default::default(),
            conn,
        }
    }

    pub fn add_tool<T: Tool>(&mut self) -> Result<()> {
        self.tools.insert(
            T::NAME,
            ToolStoreInner {
                constructor: Eraser::<T>::new(),
                description: T::DESCRIPTION,
                prompt: T::PROMPT,
                schema: serde_json::to_value(schema_for!(T::Input))
                    .context("Cannot generate schema for tool")?,
            },
        );
        Ok(())
    }

    pub fn list(&self, tool_set: ToolSet) -> (Vec<&'static str>, Vec<openrouter::Tool>) {
        tool_set
            .toold()
            .filter_map(|name| {
                self.tools.get(name).map(|tool| {
                    (
                        tool.prompt,
                        openrouter::Tool {
                            name: name.to_string(),
                            description: tool.description.to_owned(),
                            schema: tool.schema.clone(),
                        },
                    )
                })
            })
            .collect()
    }

    /// Grab a tool box
    pub async fn grab(&self, chat_id: i32, tool_set: ToolSet) -> Result<ToolBox> {
        sea_orm::ConnectionTrait::get_database_backend(&self.conn);

        let iter = tool_set
            .toold()
            .filter_map(|name| self.tools.get(name).map(|tool| (name, tool)));

        let mut tools = HashMap::new();

        for (name, inner) in iter {
            let dyn_tool = tool::Entity::find_by_id((chat_id, name.to_owned()))
                .one(&self.conn)
                .await?
                .map(|model| inner.constructor.new(&model.state))
                .transpose()?
                .unwrap_or(inner.constructor.default());

            tools.insert(name, dyn_tool);
        }

        Ok(ToolBox { tools, chat_id })
    }

    /// Put tool box back
    pub async fn put_back(&self, tool_box: ToolBox) -> Result<()> {
        for (name, dyn_tool) in tool_box.tools {
            tool::Entity::insert(tool::ActiveModel {
                chat_id: Set(tool_box.chat_id),
                function_name: Set(name.to_owned()),
                state: Set(dyn_tool.se()?),
            })
            .on_conflict(
                OnConflict::columns([tool::Column::ChatId, tool::Column::FunctionName])
                    .update_column(tool::Column::State)
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        }

        Ok(())
    }
}

trait ToolConstructor {
    fn new(&self, v: &str) -> Result<Box<dyn UntypedTool>>;
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
    fn new(&self, v: &str) -> Result<Box<dyn UntypedTool>> {
        let tool: T = serde_json::from_str(v).context("Cannot deserialize tool state")?;
        Ok(Box::new(tool))
    }

    fn default(&self) -> Box<dyn UntypedTool> {
        Box::new(T::default())
    }
}

impl ToolBox {
    pub fn get(&mut self, name: &str) -> Option<(&'static str, &mut Box<dyn UntypedTool>)> {
        let Some((name, _)) = self.tools.get_key_value(name) else {
            return None;
        };
        let name = *name;

        let tool = self.tools.get_mut(name).unwrap();
        Some((name, tool))
    }
}
