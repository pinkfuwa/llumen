use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::prelude::*;
use protocol::UserPreference;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, TransactionTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserUpdateReq {
    /// If omit will use the current user instead
    pub user_id: Option<i32>,
    pub preference: Option<UserPreference>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserUpdateResp {
    pub user_id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<UserUpdateReq>,
) -> JsonResult<UserUpdateResp> {
    let UserUpdateReq {
        user_id: user_id_req,
        preference,
        password,
    } = req;
    let user_id = user_id_req.unwrap_or(user_id);

    debug_assert!(
        preference.is_some() || password.is_some(),
        "no field to update"
    );

    let txn = app.conn.begin().await.kind(ErrorKind::Internal)?;

    let res = User::find_by_id(user_id)
        .one(&txn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or("")
        .kind(ErrorKind::ResourceNotFound)?;

    let mut active_model = res.into_active_model();

    if let Some(preference) = preference {
        let mut new_preference = active_model.preference.take().unwrap();
        if let Some(theme) = preference.theme {
            new_preference.theme = Some(theme);
        }
        if let Some(language) = preference.locale {
            new_preference.locale = Some(language);
        }
        if let Some(language) = preference.submit_on_enter {
            new_preference.submit_on_enter = Some(language);
        }
        if let Some(use_pattern_background) = preference.use_pattern_background {
            new_preference.use_pattern_background = Some(use_pattern_background);
        }
        active_model.preference = sea_orm::ActiveValue::Set(new_preference);
    }
    if let Some(password) = password {
        let password_hash = app.hasher.hash_password(&password);
        active_model.password = sea_orm::ActiveValue::Set(password_hash);
    }

    active_model.update(&txn).await.kind(ErrorKind::Internal)?;

    txn.commit().await.kind(ErrorKind::Internal)?;

    Ok(Json(UserUpdateResp { user_id }))
}
