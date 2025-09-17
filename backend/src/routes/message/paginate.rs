use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{ChunkKind, MessageKind, message, prelude::*};
use migration::ExprTrait;
use sea_orm::{QueryOrder, QuerySelect, prelude::*};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, config::MAX_PAGINATE_LIMIT, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum MessagePaginateReq {
    Limit(MessagePaginateReqLimit),
    Range(MessagePaginateReqRange),
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessagePaginateReqLimit {
    pub chat_id: i32,

    /// default to i32::MAX
    pub id: Option<i32>,
    pub order: MessagePaginateReqOrder,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[typeshare]
/// Does not include upper & lower
/// lower [... return items ... ] upper
pub struct MessagePaginateReqRange {
    pub chat_id: i32,
    pub upper: i32,
    pub lower: i32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessagePaginateReqOrder {
    /// greater than
    Gt,
    /// less than
    Lt,
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
    pub role: MessagePaginateRespRole,
    pub chunks: Vec<MessagePaginateRespChunk>,
    pub token: u32,
    pub price: f32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespChunk {
    pub id: i32,
    pub kind: MessagePaginateRespChunkKind,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessagePaginateRespRole {
    User,
    Assistant,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum MessagePaginateRespChunkKind {
    Text(MessagePaginateRespChunkKindText),
    Reasoning(MessagePaginateRespChunkKindReasoning),
    ToolCall(MessagePaginateRespChunkKindToolCall),
    Error(MessagePaginateRespChunkKindError),
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespChunkKindText {
    pub context: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespChunkKindReasoning {
    pub context: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespChunkKindToolCall {
    pub name: String,
    pub args: String,
    pub context: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessagePaginateRespChunkKindError {
    pub context: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessagePaginateReq>,
) -> JsonResult<MessagePaginateResp> {
    let q = match req {
        MessagePaginateReq::Limit(limit) => {
            let res = Chat::find_by_id(limit.chat_id)
                .one(&app.conn)
                .await
                .kind(ErrorKind::Internal)?;
            if res.is_none_or(|x| x.owner_id != user_id) {
                return Err(Json(Error {
                    error: ErrorKind::ResourceNotFound,
                    reason: "".to_owned(),
                }));
            }

            let q = Message::find()
                .filter(message::Column::ChatId.eq(limit.chat_id))
                .limit(limit.limit.unwrap_or(MAX_PAGINATE_LIMIT) as u64);
            let q = match (limit.order, limit.id) {
                (MessagePaginateReqOrder::Gt, None) => q.order_by_asc(message::Column::Id),
                (MessagePaginateReqOrder::Gt, Some(id)) => q
                    .filter(message::Column::Id.gt(id))
                    .order_by_asc(message::Column::Id),
                (MessagePaginateReqOrder::Lt, None) => q.order_by_desc(message::Column::Id),
                (MessagePaginateReqOrder::Lt, Some(id)) => q
                    .filter(message::Column::Id.lt(id))
                    .order_by_desc(message::Column::Id),
            };
            q
        }
        MessagePaginateReq::Range(range) => {
            let res = Chat::find_by_id(range.chat_id)
                .one(&app.conn)
                .await
                .kind(ErrorKind::Internal)?;
            if res.is_none_or(|x| x.owner_id != user_id) {
                return Err(Json(Error {
                    error: ErrorKind::ResourceNotFound,
                    reason: "".to_owned(),
                }));
            }

            let q = Message::find()
                .filter(message::Column::ChatId.eq(range.chat_id))
                .limit(MAX_PAGINATE_LIMIT as u64)
                .filter(message::Column::Id.gt(range.lower).lt(range.upper));
            q
        }
    };

    let res = q
        .find_with_related(Chunk)
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let list = res
        .into_iter()
        .filter_map(|(message, chunks)| {
            let role = match message.kind {
                MessageKind::User => MessagePaginateRespRole::User,
                MessageKind::Assistant => MessagePaginateRespRole::Assistant,
                MessageKind::Hidden => return None,
                MessageKind::DeepResearch => todo!("Handle DeepResearch message kind"),
            };
            let chunks: Result<_, Json<Error>> = chunks
                .into_iter()
                .map(|chunk| {
                    Ok(MessagePaginateRespChunk {
                        id: chunk.id,
                        kind: match chunk.kind {
                            ChunkKind::Text => MessagePaginateRespChunkKind::Text(
                                MessagePaginateRespChunkKindText {
                                    context: chunk.content,
                                },
                            ),
                            ChunkKind::Reasoning => MessagePaginateRespChunkKind::Reasoning(
                                MessagePaginateRespChunkKindReasoning {
                                    context: chunk.content,
                                },
                            ),
                            ChunkKind::ToolCall => {
                                let tool_call = chunk.as_tool_call().kind(ErrorKind::Internal)?;
                                MessagePaginateRespChunkKind::ToolCall(
                                    MessagePaginateRespChunkKindToolCall {
                                        name: tool_call.name,
                                        args: tool_call.args,
                                        context: tool_call.content,
                                    },
                                )
                            }
                            ChunkKind::Error => MessagePaginateRespChunkKind::Error(
                                MessagePaginateRespChunkKindError {
                                    context: chunk.content,
                                },
                            ),
                            _ => todo!("Handle other chunk kinds"),
                        },
                    })
                })
                .collect();

            Some(chunks.map(|chunks| MessagePaginateRespList {
                id: message.id,
                role,
                chunks,
                token: message.token_count as u32,
                price: message.price,
            }))
        })
        .collect::<Result<_, _>>()?;

    Ok(Json(MessagePaginateResp { list }))
}
