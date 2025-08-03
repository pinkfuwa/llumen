use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessagePaginateReq {
    /// default to i32::MAX
    /// that is, from last to first
    pub id: Option<i32>,
    pub order: MessagePaginateReqOrder,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessagePaginateReqOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateResp {
    pub list: Vec<MessagePaginateRespList>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespList {
    pub id: i32,
    pub text: String,
    pub role: MessagePaginateRespRole,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessagePaginateRespRole {
    User,
    Assistant,
    Think,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessagePaginateReq>,
) -> JsonResult<MessagePaginateResp> {
    todo!()
}
