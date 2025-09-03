mod chat;

use std::{marker::PhantomData, sync::Arc};

use anyhow::{Context, Result};
use entity::prelude::*;
use minijinja::Environment;
use sea_orm::{DbConn, EntityTrait};
use serde::Serialize;
use time::UtcDateTime;

pub use chat::ChatStore;

pub trait PromptStore {
    type Source;
    type Extra;
    type Pipe;

    async fn template(
        &self,
        locale: Option<&str>,
    ) -> Result<PromptTemplate<Self::Source, Self::Extra, Self::Pipe>>;
}

pub struct PromptTemplate<T, E = (), P = ()> {
    template: T,
    _marker: PhantomData<(P, E)>,
}

pub struct PromptEnv {
    env: Arc<Environment<'static>>,
    conn: DbConn,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptContext<E = (), P = ()> {
    pub user: UserInfo,
    pub date: UtcDateTime,
    pub chat: ChatInfo,
    pub extra: E,
    pub pipe: P,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub locale: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatInfo {
    pub id: i32,
    pub title: String,
}

impl<T, E, P> PromptTemplate<T, E, P>
where
    T: AsRef<str>,
    E: Serialize,
    P: Serialize,
{
    pub fn new(template: T) -> Self {
        Self {
            template,
            _marker: PhantomData,
        }
    }

    pub async fn render(&self, env: &PromptEnv, chat_id: i32, extra: E, pipe: P) -> Result<String> {
        let ctx = PromptContext::new(&env.conn, chat_id, extra, pipe).await?;
        let res = env.env.render_str(self.template.as_ref(), ctx)?;
        Ok(res)
    }

    pub async fn render_store() {}
}

impl PromptEnv {
    pub fn new(conn: DbConn) -> Self {
        Self {
            env: Arc::new(Environment::new()),
            conn,
        }
    }
}

impl<E, P> PromptContext<E, P> {
    pub async fn new(conn: &DbConn, chat_id: i32, extra: E, pipe: P) -> Result<Self> {
        let chat = Chat::find_by_id(chat_id)
            .one(conn)
            .await?
            .context("Cannot find chat")?;
        let user = User::find_by_id(chat.owner_id)
            .one(conn)
            .await?
            .context("Cannot find user")?;

        Ok(Self {
            user: UserInfo {
                locale: user.preference.locale.unwrap_or("en_us".to_owned()),
                name: user.name,
            },
            date: UtcDateTime::now(),
            chat: ChatInfo {
                id: chat_id,
                title: chat.title,
            },
            extra,
            pipe,
        })
    }
}
