use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Unavailable,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    /// `ok` only when every dependency is `ok`.
    pub status: Status,
    pub database: Status,
    /// The api crate version.
    #[schema(example = "0.1.0")]
    pub version: &'static str,
}

/// Reports whether api and its database are working.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = OK, description = "api and its database are working", body = Health),
        (status = SERVICE_UNAVAILABLE, description = "The database can't be reached", body = Health),
    ),
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let database = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => Status::Ok,
        Err(err) => {
            tracing::warn!(error = %err, "health check: database unavailable");
            Status::Unavailable
        }
    };
    let (code, status) = match database {
        Status::Ok => (StatusCode::OK, Status::Ok),
        Status::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, Status::Unavailable),
    };
    let body = Health {
        status,
        database,
        version: env!("CARGO_PKG_VERSION"),
    };
    (code, Json(body))
}
