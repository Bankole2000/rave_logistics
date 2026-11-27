//! The signed-in user. Clients call it on start to check that their stored session still works.

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::CurrentSession,
    error::{ApiResult, ErrorBody},
    state::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    /// Verified phone number, E.164.
    #[schema(example = "+2348031234567")]
    pub phone: String,
    pub created_at: DateTime<Utc>,
}

/// Returns the signed-in user.
#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "auth",
    security(("session" = [])),
    responses(
        (status = OK, description = "The signed-in user", body = User),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
    ),
)]
pub async fn me(State(state): State<AppState>, session: CurrentSession) -> ApiResult<Json<User>> {
    let (id, phone, created_at) =
        sqlx::query_as("SELECT id, phone, created_at FROM users WHERE id = $1")
            .bind(session.user_id)
            .fetch_one(&state.db)
            .await?;
    Ok(Json(User {
        id,
        phone,
        created_at,
    }))
}
