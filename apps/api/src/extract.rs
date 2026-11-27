//! Request extractors whose rejections use the standard error body.

use axum::extract::{FromRequest, FromRequestParts};

use crate::error::ApiError;

/// `axum::Json`, but a malformed or invalid body is rejected with the standard error body
/// (`invalid_body`) instead of axum's plain-text message.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);

/// `axum::extract::Path`, but an invalid path parameter is rejected with the standard error body
/// (`invalid_path`).
#[derive(Debug, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct ApiPath<T>(pub T);
