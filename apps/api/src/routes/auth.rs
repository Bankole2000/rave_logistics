//! Sign-in with a one-time code (FT-001, FT-002, FT-048; by email too for FT-098, FT-099) and
//! sign-out (FT-005, FT-100).
//!
//! Anyone can sign up with a phone number. Email codes are only sent to an email address that
//! already belongs to a user or has a pending staff invitation, so email is never a way to sign up.

use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{CurrentSession, destination::Destination, secrets, sender::Message},
    error::{ApiError, ApiResult, ErrorBody},
    extract::ApiJson,
    routes::me::{User, load_user},
    state::AppState,
};

/// Longest device name kept; longer names are cut.
const DEVICE_NAME_MAX_CHARS: usize = 100;

/// Send `phone` or `email`, not both.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CodeRequest {
    /// A Nigerian mobile number in any common format.
    #[schema(example = "0803 123 4567")]
    pub phone: Option<String>,
    /// For staff and vendor owners whose account or invitation has this email address.
    #[schema(example = "ada@example.com")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CodeSent {
    /// Send this back with the code.
    pub code_id: Uuid,
    /// The number the code was sent to, in E.164 form, when signing in by phone.
    #[schema(example = "+2348031234567")]
    pub phone: Option<String>,
    /// The address the code was sent to, lower-cased, when signing in by email.
    pub email: Option<String>,
    pub expires_in_seconds: i64,
    /// When another code can be requested for this number or address.
    pub resend_after_seconds: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyRequest {
    pub code_id: Uuid,
    #[schema(example = "123456")]
    pub code: String,
    /// Shown later in the list of signed-in devices, e.g. "Pixel 8". Optional.
    pub device_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignedIn {
    /// Send as `Authorization: Bearer <token>`. Shown only once; store it securely.
    pub token: String,
    pub session_id: Uuid,
    pub user: User,
    /// True when this sign-in created the account (FT-001).
    pub is_new_user: bool,
}

fn too_many(message: &str, retry_after_secs: i64) -> ApiError {
    ApiError::TooManyRequests {
        message: message.to_owned(),
        retry_after_secs,
    }
}

fn code_expired() -> ApiError {
    ApiError::BadRequest {
        code: "code_expired",
        message: "This code has expired or was already used. Ask for a new one.".to_owned(),
    }
}

fn suspended() -> ApiError {
    ApiError::Forbidden {
        code: "account_suspended",
        message: "This account is suspended. Contact support.".to_owned(),
    }
}

/// Whether an email address may receive a sign-in code: it belongs to a user or has a pending
/// staff invitation.
async fn email_can_sign_in(db: &mut PgConnection, email: &str) -> sqlx::Result<bool> {
    let (allowed,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)
             OR EXISTS (SELECT 1 FROM staff_invitations
                        WHERE email = $1 AND accepted_at IS NULL AND expires_at > now())",
    )
    .bind(email)
    .fetch_one(db)
    .await?;
    Ok(allowed)
}

/// Sends a one-time sign-in code to a phone number or an email address.
///
/// The response is the same whether or not the number or address has an account.
#[utoipa::path(
    post,
    path = "/v1/auth/codes",
    tag = "auth",
    request_body = CodeRequest,
    responses(
        (status = ACCEPTED, description = "Code sent", body = CodeSent),
        (status = BAD_REQUEST, description = "`invalid_phone`, `invalid_email` or `invalid_body`", body = ErrorBody),
        (status = TOO_MANY_REQUESTS, description = "`too_many_requests`: wait `Retry-After` seconds", body = ErrorBody,
            headers(("retry-after" = i64, description = "Seconds to wait"))),
    ),
)]
pub async fn request_code(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CodeRequest>,
) -> ApiResult<(StatusCode, Json<CodeSent>)> {
    let to = Destination::from_request(body.phone.as_deref(), body.email.as_deref())?;
    let cfg = &state.auth;
    let code_id = Uuid::new_v4();
    let response = Json(CodeSent {
        code_id,
        phone: to.phone().map(str::to_owned),
        email: to.email().map(str::to_owned),
        expires_in_seconds: cfg.code_ttl_secs,
        resend_after_seconds: cfg.code_resend_secs,
    });

    let mut tx = state.db.begin().await?;
    if let Some(email) = to.email()
        && !email_can_sign_in(&mut tx, email).await?
    {
        // Answer as if the code was sent, so the response doesn't reveal who has an account.
        tracing::info!(email = %to.masked(), "sign-in code not sent: no account or invitation");
        return Ok((StatusCode::ACCEPTED, response));
    }

    // Serialise requests for the same destination, so concurrent requests can't skip the limits.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(to.as_str())
        .execute(&mut *tx)
        .await?;
    let (sent_last_hour, secs_since_last, secs_until_slot): (i64, Option<f64>, Option<f64>) =
        sqlx::query_as(
            "SELECT count(*),
                    EXTRACT(EPOCH FROM now() - max(created_at))::float8,
                    EXTRACT(EPOCH FROM min(created_at) + interval '1 hour' - now())::float8
             FROM sign_in_codes
             WHERE destination = $1 AND created_at > now() - interval '1 hour'",
        )
        .bind(to.as_str())
        .fetch_one(&mut *tx)
        .await?;
    if let Some(elapsed) = secs_since_last {
        let wait = cfg.code_resend_secs - elapsed.floor() as i64;
        if wait > 0 {
            return Err(too_many(
                "Wait a moment before asking for another code.",
                wait,
            ));
        }
    }
    if sent_last_hour >= cfg.code_max_per_hour {
        let wait = secs_until_slot.map_or(3600, |s| s.ceil() as i64);
        return Err(too_many(
            "Too many codes for this number or address. Try again later.",
            wait,
        ));
    }

    let code = secrets::new_code();
    sqlx::query(
        "INSERT INTO sign_in_codes (id, channel, destination, code_hash, expires_at)
         VALUES ($1, $2, $3, $4, now() + make_interval(secs => $5))",
    )
    .bind(code_id)
    .bind(to.channel())
    .bind(to.as_str())
    .bind(secrets::code_hash(code_id, &code))
    .bind(cfg.code_ttl_secs as f64)
    .execute(&mut *tx)
    .await?;
    // Sent before committing: if delivery fails, the code doesn't count towards the limits.
    state
        .message_sender
        .send(&to, Message::SignInCode(&code))
        .await
        .map_err(|err| ApiError::Internal(err.context("sending a sign-in code")))?;
    tx.commit().await?;
    tracing::info!(to = %to.masked(), %code_id, "sign-in code sent");

    Ok((StatusCode::ACCEPTED, response))
}

/// Finds the user with this phone number (`sms`) or email address (`email`).
async fn find_user(
    db: &mut PgConnection,
    channel: &str,
    destination: &str,
) -> sqlx::Result<Option<(Uuid, bool)>> {
    let sql = if channel == "email" {
        "SELECT id, suspended_at IS NOT NULL FROM users WHERE email = $1"
    } else {
        "SELECT id, suspended_at IS NOT NULL FROM users WHERE phone = $1"
    };
    sqlx::query_as(sql)
        .bind(destination)
        .fetch_optional(db)
        .await
}

/// Creates a user with this verified phone number or email address. `None` if another request
/// created it first.
async fn create_user(
    db: &mut PgConnection,
    channel: &str,
    destination: &str,
) -> sqlx::Result<Option<(Uuid, DateTime<Utc>)>> {
    let sql = if channel == "email" {
        "INSERT INTO users (email) VALUES ($1) ON CONFLICT (email) DO NOTHING RETURNING id, created_at"
    } else {
        "INSERT INTO users (phone) VALUES ($1) ON CONFLICT (phone) DO NOTHING RETURNING id, created_at"
    };
    sqlx::query_as(sql)
        .bind(destination)
        .fetch_optional(db)
        .await
}

/// Checks a one-time code and signs in, creating the account on the first sign-in by phone, or
/// by email with a pending staff invitation. A pending staff invitation for the verified phone
/// number or email address is accepted (FT-072).
#[utoipa::path(
    post,
    path = "/v1/auth/sessions",
    tag = "auth",
    request_body = VerifyRequest,
    responses(
        (status = OK, description = "Signed in", body = SignedIn),
        (status = BAD_REQUEST, description = "`invalid_code`, `code_expired` or `invalid_body`", body = ErrorBody),
        (status = FORBIDDEN, description = "`account_suspended`", body = ErrorBody),
    ),
)]
pub async fn sign_in(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<VerifyRequest>,
) -> ApiResult<Json<SignedIn>> {
    let mut tx = state.db.begin().await?;
    let row: Option<(String, String, Vec<u8>, i32, bool)> = sqlx::query_as(
        "SELECT channel, destination, code_hash, attempts, used_at IS NULL AND expires_at > now()
         FROM sign_in_codes WHERE id = $1 FOR UPDATE",
    )
    .bind(body.code_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((channel, destination, stored_hash, attempts, usable)) = row else {
        return Err(code_expired());
    };
    if !usable || attempts >= state.auth.code_max_attempts {
        return Err(code_expired());
    }

    if secrets::code_hash(body.code_id, body.code.trim()) != stored_hash {
        sqlx::query("UPDATE sign_in_codes SET attempts = attempts + 1 WHERE id = $1")
            .bind(body.code_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Err(ApiError::BadRequest {
            code: "invalid_code",
            message: "That code is wrong. Check it and try again.".to_owned(),
        });
    }
    sqlx::query("UPDATE sign_in_codes SET used_at = now() WHERE id = $1")
        .bind(body.code_id)
        .execute(&mut *tx)
        .await?;

    // The phone number or email address is now verified: find its user, or create one.
    let (user_id, user_suspended, is_new_user) =
        match find_user(&mut tx, &channel, &destination).await? {
            Some((id, suspended)) => (id, suspended, false),
            None => {
                if channel == "email" && !email_can_sign_in(&mut tx, &destination).await? {
                    // The invitation expired after the code was sent.
                    return Err(code_expired());
                }
                match create_user(&mut tx, &channel, &destination).await? {
                    Some((id, _)) => (id, false, true),
                    None => {
                        let (id, suspended) = find_user(&mut tx, &channel, &destination)
                            .await?
                            .ok_or_else(code_expired)?;
                        (id, suspended, false)
                    }
                }
            }
        };

    // Accept the latest pending staff invitation for this phone number or email address.
    sqlx::query(
        "WITH accepted AS (
             UPDATE staff_invitations SET accepted_at = now(), accepted_by = $1
             WHERE id = (SELECT id FROM staff_invitations
                         WHERE (phone = $2 OR email = $2)
                           AND accepted_at IS NULL AND expires_at > now()
                         ORDER BY created_at DESC LIMIT 1)
             RETURNING role, invited_by)
         INSERT INTO staff_members (user_id, role, invited_by)
         SELECT $1, role, invited_by FROM accepted
         ON CONFLICT (user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(&destination)
    .execute(&mut *tx)
    .await?;

    let (staff_suspended,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM staff_members
                        WHERE user_id = $1 AND suspended_at IS NOT NULL)",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    if user_suspended || staff_suspended {
        tx.commit().await?;
        return Err(suspended());
    }

    let device_name = body
        .device_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(DEVICE_NAME_MAX_CHARS).collect::<String>());
    let session_id = Uuid::new_v4();
    let token = secrets::new_token();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, device_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(secrets::token_hash(&token))
    .bind(device_name)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(%user_id, %session_id, is_new_user, channel, "signed in");

    Ok(Json(SignedIn {
        token,
        session_id,
        user: load_user(&state.db, user_id).await?,
        is_new_user,
    }))
}

/// Signs out this device: its session stops working at once (FT-005, FT-100).
#[utoipa::path(
    delete,
    path = "/v1/auth/sessions/current",
    tag = "auth",
    security(("session" = [])),
    responses(
        (status = NO_CONTENT, description = "Signed out"),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
    ),
)]
pub async fn sign_out(
    State(state): State<AppState>,
    session: CurrentSession,
) -> ApiResult<StatusCode> {
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1")
        .bind(session.session_id)
        .execute(&state.db)
        .await?;
    tracing::info!(user_id = %session.user_id, session_id = %session.session_id, "signed out");
    Ok(StatusCode::NO_CONTENT)
}
