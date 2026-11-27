//! Sign-in with a phone number and a one-time code, and the sessions it creates (ADR-008).
//!
//! A session is a random bearer token held by the client; api stores only its hash, so a
//! session can be revoked at once (sign-out, suspension) by updating one row.

pub mod phone;
pub mod secrets;
pub mod sender;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// The signed-in session of the request. Handlers that take it need a valid
/// `Authorization: Bearer <token>` header; anything else is rejected with 401 `unauthorized`.
#[derive(Debug, Clone, Copy)]
pub struct CurrentSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
}

fn unauthorized() -> ApiError {
    ApiError::Unauthorized("Sign in to continue.".to_owned())
}

/// Reads the bearer token from the `Authorization` header.
fn bearer_token(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("bearer") && token.starts_with(secrets::TOKEN_PREFIX))
        .then_some(token)
}

impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = bearer_token(parts).ok_or_else(unauthorized)?;
        // One statement checks the session (not revoked, used within the idle period, user not
        // suspended) and records that it was used.
        let row: Option<(Uuid, Uuid)> = sqlx::query_as(
            "UPDATE sessions s SET last_seen_at = now()
             FROM users u
             WHERE s.token_hash = $1
               AND s.revoked_at IS NULL
               AND s.last_seen_at > now() - make_interval(days => $2)
               AND u.id = s.user_id
               AND u.suspended_at IS NULL
             RETURNING s.id, s.user_id",
        )
        .bind(secrets::token_hash(token))
        .bind(state.auth.session_idle_days)
        .fetch_optional(&state.db)
        .await?;
        let (session_id, user_id) = row.ok_or_else(unauthorized)?;
        Ok(Self {
            session_id,
            user_id,
        })
    }
}
