use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::chat;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatDeleteReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatDeleteResp {
    pub deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatDeleteReq>,
) -> JsonResult<ChatDeleteResp> {
    if req.id == 1 {
        return Err(Json(Error {
            error: ErrorKind::Internal,
            reason: "not available in demo".to_string(),
        }));
    }
    let result = chat::Entity::delete_by_id(req.id)
        .filter(chat::Column::OwnerId.eq(user_id))
        .exec(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    let deleted = result.rows_affected > 0;

    Ok(Json(ChatDeleteResp { deleted }))
}
