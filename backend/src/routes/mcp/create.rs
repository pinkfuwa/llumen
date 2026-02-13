use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::mcp_server;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, mcp::config::McpServerConfig, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct McpServerCreateReq {
    pub config_raw: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct McpServerCreateResp {
    pub id: i32,
    pub name: String,
    pub enabled: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<McpServerCreateReq>,
) -> JsonResult<McpServerCreateResp> {
    let config = McpServerConfig::parse(&req.config_raw).map_err(|e| {
        Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: format!("Invalid TOML: {e}"),
        })
    })?;

    let id = mcp_server::Entity::insert(mcp_server::ActiveModel {
        name: Set(config.name.clone()),
        config_raw: Set(req.config_raw),
        enabled: Set(config.enabled),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?
    .last_insert_id;

    Ok(Json(McpServerCreateResp {
        id,
        name: config.name,
        enabled: config.enabled,
    }))
}
