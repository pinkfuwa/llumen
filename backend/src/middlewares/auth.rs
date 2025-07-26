use std::sync::Arc;

use axum::{
    Json,
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use pasetors::{
    Local,
    token::UntrustedToken,
    version4::{LocalToken, V4},
};

use crate::{AppState, errors::*};

#[derive(Debug, Clone, Copy)]
pub struct UserId(pub u64);

pub async fn middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Json<Error>> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or("cannot find token in authorization header")
        .kind(ErrorKind::Unauthorized)?;

    let token = token.to_str().kind(ErrorKind::MalformedToken)?;
    let token = UntrustedToken::<Local, V4>::try_from(token).kind(ErrorKind::MalformedToken)?;
    let token =
        LocalToken::decrypt(&state.key, &token, None, None).kind(ErrorKind::MalformedToken)?;

    let claim = token
        .payload_claims()
        .map(|x| x.get_claim("uid").map(|x| x.as_u64()))
        .flatten()
        .flatten();

    let user_id = claim
        .ok_or("Missing claim")
        .kind(ErrorKind::MalformedToken)?;
    request.extensions_mut().insert(UserId(user_id));
    let response = next.run(request).await;

    Ok(response)
}
