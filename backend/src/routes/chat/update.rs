use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::chat;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatUpdateReq {
    pub chat_id: i32,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatUpdateResp {
    pub updated: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatUpdateReq>,
) -> JsonResult<ChatUpdateResp> {
    // TODO: sync Mode with remote

    let title = req.title.ok_or(Error {
        error: ErrorKind::MalformedRequest,
        reason: "title is required".to_string(),
    })?;

    let res = chat::Entity::update_many()
        .col_expr(chat::Column::Title, title.into())
        .filter(
            chat::Column::Id
                .eq(req.chat_id)
                .and(chat::Column::OwnerId.eq(user_id)),
        )
        .exec(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    Ok(Json(ChatUpdateResp {
        updated: res.rows_affected > 0,
    }))
}
