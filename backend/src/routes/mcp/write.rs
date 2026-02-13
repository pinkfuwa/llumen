use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::mcp_server;
use sea_orm::{ActiveValue::Set, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, mcp::config::McpServerConfig, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct McpServerWriteReq {
    pub id: i32,
    pub config_raw: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct McpServerWriteResp {
    pub id: i32,
    pub name: String,
    pub enabled: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<McpServerWriteReq>,
) -> JsonResult<McpServerWriteResp> {
    let config = McpServerConfig::parse(&req.config_raw).map_err(|e| {
        Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: format!("Invalid TOML: {e}"),
        })
    })?;

    let server = mcp_server::Entity::find_by_id(req.id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or_else(|| {
            Json(Error {
                error: ErrorKind::ResourceNotFound,
                reason: format!("MCP server id={} not found", req.id),
            })
        })?;

    // Stop running server so next tool call restarts with new config
    app.mcp.stop_server(req.id).await;

    let mut active = server.into_active_model();
    active.name = Set(config.name.clone());
    active.config_raw = Set(req.config_raw);
    active.enabled = Set(config.enabled);
    mcp_server::Entity::update(active)
        .exec(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    Ok(Json(McpServerWriteResp {
        id: req.id,
        name: config.name,
        enabled: config.enabled,
    }))
}
