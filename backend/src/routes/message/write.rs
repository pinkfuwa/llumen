use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageWriteReq {
    /// message id
    pub id: i32,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct Resp {}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessageWriteReq>,
) -> JsonResult<Resp> {
    todo!("Implement message writing logic: find the message by id, check ownership, update the message content, and save it to the database.")
}
