use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use entity::file::Entity as File;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

fn get_valid_until_timestamp() -> i32 {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    (now + 3600) as i32
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct FileRefreshReq {
    pub ids: Vec<i32>,
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
    let new_valid_until = get_valid_until_timestamp();

    let files = File::find()
        .filter(entity::file::Column::Id.is_in(req.ids))
        .filter(entity::file::Column::OwnerId.eq(user_id))
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    for file in files {
        let mut active_file: entity::file::ActiveModel = file.into();
        active_file.valid_until = Set(Some(new_valid_until));

        active_file
            .update(&app.conn)
            .await
            .kind(ErrorKind::Internal)?;
    }

    Ok(Json(FileRefreshResp {
        valid_until: new_valid_until,
    }))
}
