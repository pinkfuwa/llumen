use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::user;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserListResp {
    pub list: Vec<UserList>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserList {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserListReq {}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_): Json<UserListReq>,
) -> JsonResult<UserListResp> {
    let models = user::Entity::find()
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let list = models
        .into_iter()
        .filter_map(|m| {
            Some(UserList {
                id: m.id,
                name: m.name,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(UserListResp { list }))
}
