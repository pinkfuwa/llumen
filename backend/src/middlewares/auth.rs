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

impl Middleware {
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
            .kind(ErrorKind::MalformedToken)?;
        let user_id = i32::try_from(user_id).kind(ErrorKind::MalformedToken)?;

        Ok(UserId(user_id))
    }
}

impl FromRequestParts<Arc<AppState>> for Middleware {
    type Rejection = Json<Error>;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user_id = Self::authenticate(&parts.headers, &state.key)?;

        #[cfg(feature = "tracing")]
        {
            use tracing::info;
            info!(user_id = user_id.0, "authentication successful");
        }

        parts.extensions.insert(user_id);

        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use pasetors::claims::Claims;
    use serde_json::{Value, json};

    fn key() -> anyhow::Result<SymmetricKey<V4>> {
        Ok(SymmetricKey::from(&[7; 32])?)
    }

    fn headers_for_claims(claims: &Claims, key: &SymmetricKey<V4>) -> anyhow::Result<HeaderMap> {
        let token = local::encrypt(key, claims, None, None)?;
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_str(&token)?);
        Ok(headers)
    }

    fn headers_for_user(user_id: Value, key: &SymmetricKey<V4>) -> anyhow::Result<HeaderMap> {
        let mut claims = Claims::new()?;
        claims.add_additional("uid", user_id)?;
        headers_for_claims(&claims, key)
    }

    fn assert_malformed(headers: &HeaderMap, key: &SymmetricKey<V4>) {
        assert!(matches!(
            Middleware::authenticate(headers, key),
            Err(Json(Error {
                error: ErrorKind::MalformedToken,
                ..
            }))
        ));
    }

    #[test]
    fn authenticates_valid_tokens_without_changing_the_user_id() -> anyhow::Result<()> {
        let key = key()?;
        for user_id in [1, 42, i32::MAX] {
            let headers = headers_for_user(json!(user_id), &key)?;
            let authenticated = Middleware::authenticate(&headers, &key)
                .map_err(|error| anyhow::anyhow!("{:?}", error))?;
            assert_eq!(authenticated.0, user_id);
        }
        Ok(())
    }

    #[test]
    fn rejects_missing_authorization() -> anyhow::Result<()> {
        assert!(matches!(
            Middleware::authenticate(&HeaderMap::new(), &key()?),
            Err(Json(Error {
                error: ErrorKind::Unauthorized,
                ..
            }))
        ));
        Ok(())
    }

    #[test]
    fn rejects_non_text_and_malformed_authorization() -> anyhow::Result<()> {
        let key = key()?;
        for value in [
            HeaderValue::from_bytes(&[0xff])?,
            HeaderValue::from_static("invalid"),
            HeaderValue::from_static(""),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(header::AUTHORIZATION, value);
            assert_malformed(&headers, &key);
        }
        Ok(())
    }

    #[test]
    fn rejects_tokens_encrypted_with_another_key() -> anyhow::Result<()> {
        let headers = headers_for_user(json!(42), &key()?)?;
        assert_malformed(&headers, &SymmetricKey::from(&[8; 32])?);
        Ok(())
    }

    #[test]
    fn rejects_tokens_without_a_user_id() -> anyhow::Result<()> {
        let key = key()?;
        assert_malformed(&headers_for_claims(&Claims::new()?, &key)?, &key);
        Ok(())
    }

    #[test]
    fn rejects_non_integer_user_ids() -> anyhow::Result<()> {
        let key = key()?;
        for user_id in [
            json!("42"),
            json!(null),
            json!(true),
            json!(42.5),
            json!({"uid": 42}),
        ] {
            assert_malformed(&headers_for_user(user_id, &key)?, &key);
        }
        Ok(())
    }

    #[test]
    fn rejects_user_ids_outside_the_database_integer_range() -> anyhow::Result<()> {
        let key = key()?;
        for user_id in [
            i64::from(i32::MAX) + 1,
            i64::from(i32::MIN) - 1,
            i64::MAX,
            i64::MIN,
        ] {
            assert_malformed(&headers_for_user(json!(user_id), &key)?, &key);
        }
        Ok(())
    }

    #[test]
    fn rejects_expired_tokens() -> anyhow::Result<()> {
        let key = key()?;
        let mut claims = Claims::new()?;
        claims.add_additional("uid", 42)?;
        claims.set_expires_in(&std::time::Duration::ZERO)?;
        assert_malformed(&headers_for_claims(&claims, &key)?, &key);
        Ok(())
    }
}
