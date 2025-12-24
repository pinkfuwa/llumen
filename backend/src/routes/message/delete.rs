use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{chat, message};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageDeleteReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct Resp {
    deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessageDeleteReq>,
) -> JsonResult<Resp> {
    let (message, chat) = message::Entity::find_by_id(req.id)
        .find_also_related(chat::Entity)
        .one(&app.conn)
        .await
        .raw_kind(ErrorKind::Internal)?
        .ok_or_else(|| {
            Json(Error {
                error: ErrorKind::ResourceNotFound,
                reason: "message not found".to_owned(),
            })
        })?;

    if message.chat_id == 1 {
        return Err(Json(Error {
            error: ErrorKind::Internal,
            reason: "not available in demo".to_string(),
        }));
    }

    if chat.map(|m| m.owner_id) != Some(user_id) {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "message not found".to_owned(),
        }));
    }

    let result = message::Entity::delete_many()
        .filter(message::Column::Id.gte(req.id))
        .filter(message::Column::ChatId.eq(message.chat_id))
        .exec(&app.conn)
        .await
        .raw_kind(ErrorKind::Internal)?;

    Ok(Json(Resp {
        deleted: result.rows_affected > 0,
    }))
}
