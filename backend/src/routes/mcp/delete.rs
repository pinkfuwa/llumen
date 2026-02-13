use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::mcp_server;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct McpServerDeleteReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct McpServerDeleteResp {
    pub deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<McpServerDeleteReq>,
) -> JsonResult<McpServerDeleteResp> {
    // Stop the server if running
    app.mcp.stop_server(req.id).await;

    mcp_server::Entity::delete_by_id(req.id)
        .exec(&app.conn)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    Ok(Json(McpServerDeleteResp { deleted: true }))
}
