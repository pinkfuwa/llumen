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
    pub model_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatUpdateResp {
    pub wrote: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatUpdateReq>,
) -> JsonResult<ChatUpdateResp> {
    #[cfg(feature = "tracing")]
    {
        use tracing::info;
        info!(user_id = user_id, chat_id = req.chat_id, "updating chat");
    }

    // TODO: sync Mode with remote

    if req.title.is_none() && req.model_id.is_none() {
        return Ok(Json(ChatUpdateResp { wrote: false }));
    }

    let mut update = chat::Entity::update_many();

    if let Some(title) = req.title {
        update = update.col_expr(chat::Column::Title, title.into());
    }

    if let Some(model_id) = req.model_id {
        update = update.col_expr(chat::Column::ModelId, model_id.into());
    }

    let res = update
        .filter(
            chat::Column::Id
                .eq(req.chat_id)
                .and(chat::Column::OwnerId.eq(user_id)),
        )
        .exec(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    Ok(Json(ChatUpdateResp {
        wrote: res.rows_affected > 0,
    }))
}
