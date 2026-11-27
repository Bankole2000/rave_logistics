//! Sign-in with a phone number and a one-time code, and the sessions it creates (ADR-008).
//!
//! A session is a random bearer token held by the client; api stores only its hash, so a
//! session can be revoked at once (sign-out, suspension) by updating one row.

pub mod destination;
pub mod phone;
pub mod roles;
pub mod secrets;
pub mod sender;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::{
    auth::roles::{Permission, Role},
    error::ApiError,
    state::AppState,
};

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

/// A signed-in staff member who isn't suspended. Anyone else is rejected: 401 `unauthorized`
/// without a valid session, 403 `not_staff` with one. Check what the role may do with
/// [`CurrentStaff::require`].
#[derive(Debug, Clone, Copy)]
pub struct CurrentStaff {
    pub session: CurrentSession,
    pub role: Role,
}

impl CurrentStaff {
    /// 403 `forbidden` unless the role has `permission`.
    pub fn require(&self, permission: Permission) -> Result<(), ApiError> {
        if self.role.can(permission) {
            Ok(())
        } else {
            Err(ApiError::Forbidden {
                code: "forbidden",
                message: "Your role can't do this.".to_owned(),
            })
        }
    }
}

impl FromRequestParts<AppState> for CurrentStaff {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let session = CurrentSession::from_request_parts(parts, state).await?;
        let role: Option<(String,)> = sqlx::query_as(
            "SELECT role FROM staff_members WHERE user_id = $1 AND suspended_at IS NULL",
        )
        .bind(session.user_id)
        .fetch_optional(&state.db)
        .await?;
        let role =
            role.and_then(|(r,)| Role::from_stored(&r))
                .ok_or_else(|| ApiError::Forbidden {
                    code: "not_staff",
                    message: "This needs a staff account.".to_owned(),
                })?;
        Ok(Self { session, role })
    }
}
