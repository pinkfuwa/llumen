use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatPaginateReq {
    /// default to i32::MAX
    /// that is, from last to first
    pub id: Option<i32>,
    pub order: ChatPaginateReqOrder,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum ChatPaginateReqOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatPaginateResp {
    pub list: Vec<ChatPaginateRespList>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatPaginateRespList {
    pub id: i32,
    pub model_id: i32,
    pub title: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatPaginateReq>,
) -> JsonResult<ChatPaginateResp> {
    todo!()
}
