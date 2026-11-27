//! The standard error body. Every error response from api has this shape:
//!
//! ```json
//! { "error": { "code": "not_found", "message": "No route for GET /nope" } }
//! ```
//!
//! `code` is stable and meant for clients to branch on; `message` is for people and may change.
//! The request ID is in the `x-request-id` response header.

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    /// Stable, machine-readable error code, e.g. `not_found`.
    #[schema(example = "not_found")]
    pub code: String,
    /// Human-readable explanation. Not stable; don't parse it.
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{message}")]
    BadRequest { code: &'static str, message: String },
    /// Missing, invalid, expired or revoked session. Sent with `WWW-Authenticate: Bearer`.
    #[error("{0}")]
    Unauthorized(String),
    #[error("{message}")]
    Forbidden { code: &'static str, message: String },
    #[error("{0}")]
    NotFound(String),
    /// A rate limit was hit. Sent with `Retry-After` in seconds.
    #[error("{message}")]
    TooManyRequests {
        message: String,
        retry_after_secs: i64,
    },
    #[error("{0}")]
    ServiceUnavailable(String),
    /// Unexpected failure. The cause is logged, never sent to the client.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::BadRequest { code, .. } => (StatusCode::BAD_REQUEST, code),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden { code, .. } => (StatusCode::FORBIDDEN, code),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::TooManyRequests { .. } => (StatusCode::TOO_MANY_REQUESTS, "too_many_requests"),
            Self::ServiceUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::BadRequest {
            code: "invalid_body",
            message: rejection.body_text(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let extra_header = match &self {
            Self::Unauthorized(_) => {
                Some((header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer")))
            }
            Self::TooManyRequests {
                retry_after_secs, ..
            } => HeaderValue::from_str(&retry_after_secs.max(&1).to_string())
                .ok()
                .map(|v| (header::RETRY_AFTER, v)),
            _ => None,
        };
        let message = match &self {
            Self::Internal(err) => {
                tracing::error!(error = ?err, "internal error");
                "Something went wrong on our side.".to_owned()
            }
            other => other.to_string(),
        };
        let body = ErrorBody {
            error: ErrorDetail {
                code: code.to_owned(),
                message,
            },
        };
        let mut response = (status, Json(body)).into_response();
        if let Some((name, value)) = extra_header {
            response.headers_mut().insert(name, value);
        }
        response
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
