use crate::{
    AppState,
    errors::{AppError, ErrorKind, WithKind},
};
use pasetors::{claims::Claims, local};
use std::time::Duration;

pub struct Token {
    pub token: String,
    pub exp: String,
}
pub fn new_token(app: &AppState, user_id: i32) -> Result<Token, AppError> {
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

    Ok(Token { token, exp })
}
