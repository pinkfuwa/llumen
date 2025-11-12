use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{chat, model};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, utils::chat::ChatMode};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatReadReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatReadResp {
    pub mode: ChatMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatReadReq>,
) -> JsonResult<ChatReadResp> {
    #[cfg(feature = "tracing")]
    {
        use tracing::info;
        info!(user_id = user_id, chat_id = req.id, "reading chat");
    }

    let res = chat::Entity::find_by_id(req.id)
        .filter(chat::Column::OwnerId.eq(user_id))
        .find_also_related(model::Entity)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    match res {
        Some((chat, model)) => Ok(Json(ChatReadResp {
            model_id: model.map(|x| x.id),
            mode: chat.mode.into(),
            title: chat.title,
        })),
        None => {
            return Err(Json(Error {
                error: ErrorKind::ResourceNotFound,
                reason: "".to_owned(),
            }));
        }
    }
}
