use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use entity::file::Entity as File;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

fn get_valid_until_timestamp() -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    (now + 3600) as i32
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct FileRefreshReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct FileRefreshResp {
    pub valid_until: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<FileRefreshReq>,
) -> JsonResult<FileRefreshResp> {
    let file = File::find_by_id(req.id)
        .filter(entity::file::Column::OwnerId.eq(user_id))
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    if file.is_none() {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "file not found or access denied".into(),
        }));
    }

    let file = file.unwrap();

    let new_valid_until = get_valid_until_timestamp();

    let mut active_file: entity::file::ActiveModel = file.into();
    active_file.valid_until = Set(Some(new_valid_until));

    active_file
        .update(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    Ok(Json(FileRefreshResp {
        valid_until: new_valid_until,
    }))
}
