//! The signed-in user. Clients call it on start to check that their stored session still works,
//! and staff apps use `staff.permissions` to show only the tools of the role (FT-072).

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{
        CurrentSession,
        roles::{Permission, Role},
    },
    error::{ApiError, ApiResult, ErrorBody},
    state::AppState,
};

#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    /// Verified phone number, E.164. A user has a phone number, an email address, or both.
    #[schema(example = "+2348031234567")]
    pub phone: Option<String>,
    /// Verified email address, lower-cased.
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Present when the user is an internal staff member.
    pub staff: Option<StaffAccess>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StaffAccess {
    pub role: Role,
    /// What the role may do. The server checks these on every request.
    pub permissions: Vec<Permission>,
    /// A suspended staff member can't sign in (FT-073).
    pub suspended: bool,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    phone: Option<String>,
    email: Option<String>,
    created_at: DateTime<Utc>,
    role: Option<String>,
    staff_suspended: Option<bool>,
}

/// Loads a user with their staff access.
pub async fn load_user(db: &PgPool, id: Uuid) -> ApiResult<User> {
    let UserRow {
        phone,
        email,
        created_at,
        role,
        staff_suspended,
    } = sqlx::query_as(
        "SELECT u.phone, u.email, u.created_at, s.role, s.suspended_at IS NOT NULL AS staff_suspended
         FROM users u LEFT JOIN staff_members s ON s.user_id = u.id
         WHERE u.id = $1",
    )
    .bind(id)
    .fetch_one(db)
    .await?;
    let staff = match role {
        None => None,
        Some(role) => {
            let role = Role::from_stored(&role).ok_or_else(|| {
                ApiError::Internal(anyhow::anyhow!("unknown staff role {role:?} for user {id}"))
            })?;
            Some(StaffAccess {
                role,
                permissions: role.permissions().to_vec(),
                suspended: staff_suspended.unwrap_or(false),
            })
        }
    };
    Ok(User {
        id,
        phone,
        email,
        created_at,
        staff,
    })
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
    Ok(Json(load_user(&state.db, session.user_id).await?))
}
