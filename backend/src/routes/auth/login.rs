use std::sync::Arc;

use axum::{Json, extract::State};
use entity::{prelude::*, user};
use pasetors::{claims::Claims, local};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct LoginResp {
    pub token: String,
    pub exp: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Json(req): Json<LoginReq>,
) -> JsonResult<LoginResp> {
    let model = User::find()
        .filter(user::Column::Name.eq(req.username))
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or("")
        .kind(ErrorKind::UnknownUser)?;

    if model.password != req.password {
        return Err(Json(Error {
            error: ErrorKind::UnknownUser,
            reason: "".to_owned(),
        }));
    }

    let mut claim = Claims::new().kind(ErrorKind::Internal)?;

    // safety:
    // "uid" is not reserve
    claim.add_additional("uid", model.id).unwrap();

    // safety:
    // "exp" must exists
    let exp = claim.get_claim("exp").unwrap().as_str().unwrap().to_owned();

    let token = local::encrypt(&app.key, &claim, None, None).kind(ErrorKind::Internal)?;

    Ok(Json(LoginResp { token, exp }))
}
