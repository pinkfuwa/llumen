use std::sync::Arc;

use axum::{
    Json,
    extract::FromRequestParts,
    http::{HeaderMap, header, request::Parts},
};
use pasetors::{
    Local, claims::ClaimsValidationRules, keys::SymmetricKey, local, token::UntrustedToken,
    version4::V4,
};

use crate::{AppState, errors::*};

#[derive(Debug, Clone, Copy)]
pub struct UserId(pub i32);

pub struct Middleware;

fn authenticate(headers: &HeaderMap, key: &SymmetricKey<V4>) -> Result<UserId, Json<Error>> {
    let token = headers
        .get(header::AUTHORIZATION)
        .ok_or("cannot find token in authorization header")
        .kind(ErrorKind::Unauthorized)?;

    let token = token.to_str().kind(ErrorKind::MalformedToken)?;
    let token = UntrustedToken::<Local, V4>::try_from(token).kind(ErrorKind::MalformedToken)?;
    let validation_rules = ClaimsValidationRules::new();
    let token = local::decrypt(key, &token, &validation_rules, None, None)
        .kind(ErrorKind::MalformedToken)?;

    let user_id = token
        .payload_claims()
        .and_then(|claims| claims.get_claim("uid"))
        .and_then(|claim| claim.as_i64())
        .ok_or("Missing claim")
        .kind(ErrorKind::MalformedToken)? as i32;

    Ok(UserId(user_id))
}

impl FromRequestParts<Arc<AppState>> for Middleware {
    type Rejection = Json<Error>;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user_id = authenticate(&parts.headers, &state.key)?;

        #[cfg(feature = "tracing")]
        {
            use tracing::info;
            info!(user_id = user_id.0, "authentication successful");
        }

        parts.extensions.insert(user_id);

        Ok(Self)
    }
}
