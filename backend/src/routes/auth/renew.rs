use std::{sync::Arc, time::Duration};

use axum::{Json, extract::State};
use pasetors::{
    Local,
    claims::{Claims, ClaimsValidationRules},
    local,
    token::UntrustedToken,
    version4::V4,
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
    pub exp: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Json(RenewReq { token }): Json<RenewReq>,
) -> JsonResult<RenewResp> {
    let token = UntrustedToken::<Local, V4>::try_from(&token).kind(ErrorKind::MalformedRequest)?;

    let token = local::decrypt(&app.key, &token, &ClaimsValidationRules::new(), None, None)
        .kind(ErrorKind::MalformedRequest)?;
    let claim = token
        .payload_claims()
        .map(|x| x.get_claim("uid").map(|x| x.as_u64()))
        .flatten()
        .flatten();

    let user_id = claim
        .ok_or("Cannot get user id")
        .kind(ErrorKind::MalformedRequest)?;
    let mut claim = Claims::new().kind(ErrorKind::Internal)?;
    let expiration = Duration::from_secs(60 * 60 * 24 * 7);
    claim
        .set_expires_in(&expiration)
        .kind(ErrorKind::Internal)?;

    // safety:
    // "uid" is not reserve
    claim.add_additional("uid", user_id).unwrap();

    // safety:
    // "exp" must exists
    let exp = claim.get_claim("exp").unwrap().as_str().unwrap().to_owned();

    let token = local::encrypt(&app.key, &claim, None, None).kind(ErrorKind::Internal)?;

    Ok(Json(RenewResp { token, exp }))
}
