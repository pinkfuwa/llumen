use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{chat, prelude::*};
use sea_orm::{QueryOrder, QuerySelect, prelude::*};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, config::MAX_PAGINATE_LIMIT, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum ChatPaginateReq {
    Limit(ChatPaginateReqLimit),
    Range(ChatPaginateReqRange),
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatPaginateReqLimit {
    /// Default to the beginning
    /// For Gt => minimum id
    /// For Le => maximum id
    pub id: Option<i32>,
    pub order: ChatPaginateReqOrder,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[typeshare]
/// Does not include upper & lower
/// lower [... return items ... ] upper
pub struct ChatPaginateReqRange {
    pub upper: i32,
    pub lower: i32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum ChatPaginateReqOrder {
    /// greater than
    GT,
    /// less than
    LT,
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
    let q = match req {
        ChatPaginateReq::Limit(limit) => {
            let q = Chat::find()
                .filter(chat::Column::OwnerId.eq(user_id))
                .limit(
                    limit
                        .limit
                        .map(|x| x.min(MAX_PAGINATE_LIMIT))
                        .unwrap_or(MAX_PAGINATE_LIMIT) as u64,
                );
            let q = match (limit.order, limit.id) {
                (ChatPaginateReqOrder::GT, None) => q.order_by_asc(chat::Column::Id),
                (ChatPaginateReqOrder::GT, Some(id)) => q
                    .filter(chat::Column::Id.gt(id))
                    .order_by_asc(chat::Column::Id),
                (ChatPaginateReqOrder::LT, None) => q.order_by_desc(chat::Column::Id),
                (ChatPaginateReqOrder::LT, Some(id)) => q
                    .filter(chat::Column::Id.lt(id))
                    .order_by_desc(chat::Column::Id),
            };
            q
        }
        ChatPaginateReq::Range(range) => Chat::find()
            .filter(chat::Column::OwnerId.eq(user_id))
            .filter(chat::Column::Id.gt(range.lower))
            .filter(chat::Column::Id.lt(range.upper))
            .limit(MAX_PAGINATE_LIMIT as u64),
    };

    let list = q
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .into_iter()
        .map(|x| ChatPaginateRespList {
            id: x.id,
            model_id: x.model_id,
            title: x.title,
        })
        .collect();
    Ok(Json(ChatPaginateResp { list }))
}
