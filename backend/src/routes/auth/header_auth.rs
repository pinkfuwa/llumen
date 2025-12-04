use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap};
use entity::{prelude::*, user};
use sea_orm::prelude::*;
use serde::Serialize;
use typeshare::typeshare;

use super::helper;
use crate::{AppState, errors::*};

#[derive(Debug, Clone, Serialize)]
#[typeshare]
pub struct HeaderAuthResp {
    pub token: Option<String>,
    pub exp: Option<String>,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    headers: HeaderMap,
) -> JsonResult<HeaderAuthResp> {
    let header = app.auth_header.as_deref();
    let username = header.and_then(|x| headers.get(x).and_then(|x| x.to_str().ok()));

    if username.is_none() {
        return Ok(Json(HeaderAuthResp {
            token: None,
            exp: None,
        }));
    }

    let model = User::find()
        .filter(user::Column::Name.eq(username.unwrap()))
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or("User not found")
        .kind(ErrorKind::ResourceNotFound)?;

    let helper::Token { token, exp } = helper::new_token(&app, model.id)?;

    Ok(Json(HeaderAuthResp {
        token: Some(token),
        exp: Some(exp),
    }))
}
