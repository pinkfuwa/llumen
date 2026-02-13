use axum::{Extension, Json};

use crate::{errors::*, mcp::config::McpServerConfig, middlewares::auth::UserId};

#[derive(serde::Deserialize, serde::Serialize)]
#[typeshare::typeshare]
pub struct McpCheckReq {
    pub config_raw: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[typeshare::typeshare]
pub struct McpCheckResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub async fn route(
    Extension(UserId(_user_id)): Extension<UserId>,
    Json(req): Json<McpCheckReq>,
) -> JsonResult<McpCheckResp> {
    match McpServerConfig::parse(&req.config_raw) {
        Ok(_) => Ok(Json(McpCheckResp { reason: None })),
        Err(e) => Ok(Json(McpCheckResp {
            reason: Some(e.to_string()),
        })),
    }
}
