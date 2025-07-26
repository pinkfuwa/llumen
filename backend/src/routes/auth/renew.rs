use std::sync::Arc;

use axum::{Json, extract::State};
use pasetors::{
    Local,
    claims::Claims,
    local,
    token::UntrustedToken,
    version4::{LocalToken, V4},
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*};

#[derive(Debug, Clone, Deserialize)]
#[typeshare]
pub struct RenewReq {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
#[typeshare]
pub struct RenewResp {
    pub token: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Json(RenewReq { token }): Json<RenewReq>,
) -> JsonResult<RenewResp> {
    let token = UntrustedToken::<Local, V4>::try_from(&token).kind(ErrorKind::MalformedRequest)?;

    let token =
        LocalToken::decrypt(&app.key, &token, None, None).kind(ErrorKind::MalformedRequest)?;
    let claim = token
        .payload_claims()
        .map(|x| x.get_claim("uid").map(|x| x.as_u64()))
        .flatten()
        .flatten();

    let user_id = claim
        .ok_or("Cannot get user id")
        .kind(ErrorKind::MalformedRequest)?;
    let mut claim = Claims::new().kind(ErrorKind::Internal)?;

    // safety:
    // "uid" is not reserve
    claim.add_additional("uid", user_id).unwrap();

    let token = local::encrypt(&app.key, &claim, None, None).kind(ErrorKind::Internal)?;

    Ok(Json(RenewResp { token }))
}
