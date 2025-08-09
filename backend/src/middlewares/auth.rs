use std::sync::Arc;

use axum::{
    Json,
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts},
    middleware::Next,
    response::Response,
};
use pasetors::{Local, claims::ClaimsValidationRules, local, token::UntrustedToken, version4::V4};

use crate::{AppState, errors::*};

#[derive(Debug, Clone, Copy)]
pub struct UserId(pub i32);

pub struct Middleware;

impl FromRequestParts<Arc<AppState>> for Middleware {
    type Rejection = Json<Error>;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or("cannot find token in authorization header")
            .kind(ErrorKind::Unauthorized)?;

        let token = token.to_str().kind(ErrorKind::MalformedToken)?;
        let token = UntrustedToken::<Local, V4>::try_from(token).kind(ErrorKind::MalformedToken)?;
        let validation_rules = ClaimsValidationRules::new();
        let token = local::decrypt(&state.key, &token, &validation_rules, None, None)
            .kind(ErrorKind::MalformedToken)?;

        let claim = token
            .payload_claims()
            .map(|x| x.get_claim("uid").map(|x| x.as_i64()))
            .flatten()
            .flatten();

        let user_id = claim
            .ok_or("Missing claim")
            .kind(ErrorKind::MalformedToken)? as i32;
        parts.extensions.insert(UserId(user_id));

        Ok(Self)
    }
}
