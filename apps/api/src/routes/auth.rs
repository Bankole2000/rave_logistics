//! Sign-in with a phone number and a one-time code (FT-001, FT-002, FT-048), and sign-out (FT-005).

use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{CurrentSession, phone::Phone, secrets},
    error::{ApiError, ApiResult, ErrorBody},
    extract::ApiJson,
    routes::me::User,
    state::AppState,
};

/// Longest device name kept; longer names are cut.
const DEVICE_NAME_MAX_CHARS: usize = 100;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CodeRequest {
    /// A Nigerian mobile number in any common format.
    #[schema(example = "0803 123 4567")]
    pub phone: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CodeSent {
    /// Send this back with the code.
    pub code_id: Uuid,
    /// The number the code was sent to, in E.164 form.
    #[schema(example = "+2348031234567")]
    pub phone: String,
    pub expires_in_seconds: i64,
    /// When another code can be requested for this number.
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

/// Sends a one-time sign-in code to a phone number.
///
/// The response is the same whether or not the number has an account.
#[utoipa::path(
    post,
    path = "/v1/auth/codes",
    tag = "auth",
    request_body = CodeRequest,
    responses(
        (status = ACCEPTED, description = "Code sent", body = CodeSent),
        (status = BAD_REQUEST, description = "`invalid_phone` or `invalid_body`", body = ErrorBody),
        (status = TOO_MANY_REQUESTS, description = "`too_many_requests`: wait `Retry-After` seconds", body = ErrorBody,
            headers(("retry-after" = i64, description = "Seconds to wait"))),
    ),
)]
pub async fn request_code(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CodeRequest>,
) -> ApiResult<(StatusCode, Json<CodeSent>)> {
    let phone = Phone::parse(&body.phone).ok_or_else(|| ApiError::BadRequest {
        code: "invalid_phone",
        message: "Enter a Nigerian mobile number, e.g. 0803 123 4567.".to_owned(),
    })?;
    let cfg = &state.auth;

    let mut tx = state.db.begin().await?;
    // Serialise requests for the same number, so concurrent requests can't skip the limits.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(phone.as_str())
        .execute(&mut *tx)
        .await?;
    let (sent_last_hour, secs_since_last, secs_until_slot): (i64, Option<f64>, Option<f64>) =
        sqlx::query_as(
            "SELECT count(*),
                    EXTRACT(EPOCH FROM now() - max(created_at))::float8,
                    EXTRACT(EPOCH FROM min(created_at) + interval '1 hour' - now())::float8
             FROM sign_in_codes
             WHERE phone = $1 AND created_at > now() - interval '1 hour'",
        )
        .bind(phone.as_str())
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
            "Too many codes for this number. Try again later.",
            wait,
        ));
    }

    let code_id = Uuid::new_v4();
    let code = secrets::new_code();
    sqlx::query(
        "INSERT INTO sign_in_codes (id, phone, code_hash, expires_at)
         VALUES ($1, $2, $3, now() + make_interval(secs => $4))",
    )
    .bind(code_id)
    .bind(phone.as_str())
    .bind(secrets::code_hash(code_id, &code))
    .bind(cfg.code_ttl_secs as f64)
    .execute(&mut *tx)
    .await?;
    // Sent before committing: if delivery fails, the code doesn't count towards the limits.
    state
        .code_sender
        .send(&phone, &code)
        .await
        .map_err(|err| ApiError::Internal(err.context("sending a sign-in code")))?;
    tx.commit().await?;
    tracing::info!(phone = %phone.masked(), %code_id, "sign-in code sent");

    Ok((
        StatusCode::ACCEPTED,
        Json(CodeSent {
            code_id,
            phone: phone.to_string(),
            expires_in_seconds: cfg.code_ttl_secs,
            resend_after_seconds: cfg.code_resend_secs,
        }),
    ))
}

/// Checks a one-time code and signs in, creating the account on first sign-in.
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
    let row: Option<(String, Vec<u8>, i32, bool)> = sqlx::query_as(
        "SELECT phone, code_hash, attempts, used_at IS NULL AND expires_at > now()
         FROM sign_in_codes WHERE id = $1 FOR UPDATE",
    )
    .bind(body.code_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((phone, stored_hash, attempts, usable)) = row else {
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

    // The number is now verified: find its user, or create one (FT-001).
    let created: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(
        "INSERT INTO users (phone) VALUES ($1) ON CONFLICT (phone) DO NOTHING
         RETURNING id, created_at",
    )
    .bind(&phone)
    .fetch_optional(&mut *tx)
    .await?;
    let is_new_user = created.is_some();
    let (user_id, created_at, suspended) = match created {
        Some((id, at)) => (id, at, false),
        None => {
            sqlx::query_as(
                "SELECT id, created_at, suspended_at IS NOT NULL FROM users WHERE phone = $1",
            )
            .bind(&phone)
            .fetch_one(&mut *tx)
            .await?
        }
    };
    if suspended {
        tx.commit().await?;
        return Err(ApiError::Forbidden {
            code: "account_suspended",
            message: "This account is suspended. Contact support.".to_owned(),
        });
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
    tracing::info!(%user_id, %session_id, is_new_user, "signed in");

    Ok(Json(SignedIn {
        token,
        session_id,
        user: User {
            id: user_id,
            phone,
            created_at,
        },
        is_new_user,
    }))
}

/// Signs out this device: its session stops working at once (FT-005).
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
