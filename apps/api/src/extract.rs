//! Request extractors whose rejections use the standard error body.

use axum::extract::FromRequest;

use crate::error::ApiError;

/// `axum::Json`, but a malformed or invalid body is rejected with the standard error body
/// (`invalid_body`) instead of axum's plain-text message.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);
