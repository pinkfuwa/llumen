use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::mcp_server;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, mcp::config::McpServerConfig, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct McpServerListReq {}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct McpServerListResp {
    pub list: Vec<McpServerListItem>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct McpServerListItem {
    pub id: i32,
    pub name: String,
    pub enabled: bool,
    pub transport: String,
    pub running: bool,
    pub config_raw: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_): Json<McpServerListReq>,
) -> JsonResult<McpServerListResp> {
    let servers = mcp_server::Entity::find()
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    let mut list = Vec::new();
    for s in servers {
        let transport = McpServerConfig::parse(&s.config_raw)
            .map(|c| c.transport.to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let running = app.mcp.is_running(s.id).await;

        list.push(McpServerListItem {
            id: s.id,
            name: s.name,
            enabled: s.enabled,
            transport,
            running,
            config_raw: s.config_raw,
        });
    }

    Ok(Json(McpServerListResp { list }))
}
