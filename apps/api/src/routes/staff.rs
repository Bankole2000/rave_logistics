//! Staff accounts: invitations with a role (FT-072) and suspension (FT-073). Admins only.

use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::{
        CurrentStaff,
        destination::Destination,
        roles::{Permission, Role},
        sender::Message,
    },
    error::{ApiError, ApiResult, ErrorBody},
    extract::{ApiJson, ApiPath},
    state::AppState,
};

/// Invite by `phone` or `email`, not both.
#[derive(Debug, Deserialize, ToSchema)]
pub struct InvitationRequest {
    #[schema(example = "0803 123 4567")]
    pub phone: Option<String>,
    #[schema(example = "ada@example.com")]
    pub email: Option<String>,
    pub role: Role,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Invitation {
    pub id: Uuid,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: Role,
    /// The admin who sent it; empty for the first admin's invitation.
    pub invited_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StaffMember {
    pub user_id: Uuid,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    /// When the staff member was suspended; empty if active.
    pub suspended_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct StaffRow {
    user_id: Uuid,
    phone: Option<String>,
    email: Option<String>,
    role: String,
    created_at: DateTime<Utc>,
    suspended_at: Option<DateTime<Utc>>,
}

fn role(stored: &str) -> ApiResult<Role> {
    Role::from_stored(stored)
        .ok_or_else(|| ApiError::Internal(anyhow::anyhow!("unknown staff role {stored:?}")))
}

type InvitationRow = (
    Uuid,
    Option<String>,
    Option<String>,
    String,
    Option<Uuid>,
    DateTime<Utc>,
    DateTime<Utc>,
);

fn invitation(
    (id, phone, email, r, invited_by, created_at, expires_at): InvitationRow,
) -> ApiResult<Invitation> {
    Ok(Invitation {
        id,
        phone,
        email,
        role: role(&r)?,
        invited_by,
        created_at,
        expires_at,
    })
}

/// Creates a pending invitation and replaces any earlier pending one for the same destination.
/// Shared with the `invite-admin` command, which passes no inviter.
pub async fn create_invitation(
    db: &mut sqlx::PgConnection,
    to: &Destination,
    role: Role,
    invited_by: Option<Uuid>,
    valid_days: i32,
) -> ApiResult<Invitation> {
    let (already_staff,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM staff_members s JOIN users u ON u.id = s.user_id
                        WHERE u.phone = $1 OR u.email = $1)",
    )
    .bind(to.as_str())
    .fetch_one(&mut *db)
    .await?;
    if already_staff {
        return Err(ApiError::Conflict {
            code: "already_staff",
            message: "This person is already a staff member.".to_owned(),
        });
    }
    sqlx::query(
        "UPDATE staff_invitations SET expires_at = now()
         WHERE (phone = $1 OR email = $1) AND accepted_at IS NULL AND expires_at > now()",
    )
    .bind(to.as_str())
    .execute(&mut *db)
    .await?;
    let row: InvitationRow = sqlx::query_as(
        "INSERT INTO staff_invitations (id, phone, email, role, invited_by, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(days => $6))
         RETURNING id, phone, email, role, invited_by, created_at, expires_at",
    )
    .bind(Uuid::new_v4())
    .bind(to.phone())
    .bind(to.email())
    .bind(role.as_str())
    .bind(invited_by)
    .bind(valid_days)
    .fetch_one(&mut *db)
    .await?;
    invitation(row)
}

/// Invites someone to join as staff with a role (FT-072). They accept by signing in with the
/// invited phone number or email address before the invitation expires.
#[utoipa::path(
    post,
    path = "/v1/staff/invitations",
    tag = "staff",
    security(("session" = [])),
    request_body = InvitationRequest,
    responses(
        (status = CREATED, description = "Invitation sent", body = Invitation),
        (status = BAD_REQUEST, description = "`invalid_phone`, `invalid_email` or `invalid_body`", body = ErrorBody),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
        (status = FORBIDDEN, description = "`not_staff` or `forbidden` (needs `staff.create`)", body = ErrorBody),
        (status = CONFLICT, description = "`already_staff`", body = ErrorBody),
    ),
)]
pub async fn invite(
    State(state): State<AppState>,
    staff: CurrentStaff,
    ApiJson(body): ApiJson<InvitationRequest>,
) -> ApiResult<(StatusCode, Json<Invitation>)> {
    staff.require(Permission::StaffCreate)?;
    let to = Destination::from_request(body.phone.as_deref(), body.email.as_deref())?;

    let mut tx = state.db.begin().await?;
    let invitation = create_invitation(
        &mut tx,
        &to,
        body.role,
        Some(staff.session.user_id),
        state.auth.invitation_days,
    )
    .await?;
    state
        .message_sender
        .send(&to, Message::StaffInvitation(body.role))
        .await
        .map_err(|err| ApiError::Internal(err.context("sending a staff invitation")))?;
    tx.commit().await?;
    tracing::info!(
        to = %to.masked(),
        role = body.role.as_str(),
        invited_by = %staff.session.user_id,
        invitation_id = %invitation.id,
        "staff invitation sent"
    );
    Ok((StatusCode::CREATED, Json(invitation)))
}

/// Lists pending invitations, newest first.
#[utoipa::path(
    get,
    path = "/v1/staff/invitations",
    tag = "staff",
    security(("session" = [])),
    responses(
        (status = OK, description = "Pending invitations", body = Vec<Invitation>),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
        (status = FORBIDDEN, description = "`not_staff` or `forbidden` (needs `staff.create`)", body = ErrorBody),
    ),
)]
pub async fn list_invitations(
    State(state): State<AppState>,
    staff: CurrentStaff,
) -> ApiResult<Json<Vec<Invitation>>> {
    staff.require(Permission::StaffCreate)?;
    let rows: Vec<InvitationRow> = sqlx::query_as(
        "SELECT id, phone, email, role, invited_by, created_at, expires_at
         FROM staff_invitations
         WHERE accepted_at IS NULL AND expires_at > now()
         ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter().map(invitation).collect::<ApiResult<_>>()?,
    ))
}

/// Lists staff members, newest first.
#[utoipa::path(
    get,
    path = "/v1/staff",
    tag = "staff",
    security(("session" = [])),
    responses(
        (status = OK, description = "Staff members", body = Vec<StaffMember>),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
        (status = FORBIDDEN, description = "`not_staff` or `forbidden` (needs `staff.view`)", body = ErrorBody),
    ),
)]
pub async fn list_staff(
    State(state): State<AppState>,
    staff: CurrentStaff,
) -> ApiResult<Json<Vec<StaffMember>>> {
    staff.require(Permission::StaffView)?;
    let rows: Vec<StaffRow> = sqlx::query_as(
        "SELECT s.user_id, u.phone, u.email, s.role, s.created_at, s.suspended_at
         FROM staff_members s JOIN users u ON u.id = s.user_id
         ORDER BY s.created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;
    let members = rows
        .into_iter()
        .map(|row| {
            Ok(StaffMember {
                user_id: row.user_id,
                phone: row.phone,
                email: row.email,
                role: role(&row.role)?,
                created_at: row.created_at,
                suspended_at: row.suspended_at,
            })
        })
        .collect::<ApiResult<_>>()?;
    Ok(Json(members))
}

/// Suspends a staff member: every session of theirs ends at once and they can't sign in (FT-073).
#[utoipa::path(
    post,
    path = "/v1/staff/{user_id}/suspension",
    tag = "staff",
    security(("session" = [])),
    params(("user_id" = Uuid, Path, description = "The staff member's user ID")),
    responses(
        (status = NO_CONTENT, description = "Suspended (also when already suspended)"),
        (status = BAD_REQUEST, description = "`cannot_suspend_self`", body = ErrorBody),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
        (status = FORBIDDEN, description = "`not_staff` or `forbidden` (needs `staff.suspend`)", body = ErrorBody),
        (status = NOT_FOUND, description = "`not_found`: no such staff member", body = ErrorBody),
    ),
)]
pub async fn suspend(
    State(state): State<AppState>,
    staff: CurrentStaff,
    ApiPath(user_id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    staff.require(Permission::StaffSuspend)?;
    if user_id == staff.session.user_id {
        return Err(ApiError::BadRequest {
            code: "cannot_suspend_self",
            message: "You can't suspend your own account.".to_owned(),
        });
    }
    let mut tx = state.db.begin().await?;
    let found = sqlx::query(
        "UPDATE staff_members SET suspended_at = coalesce(suspended_at, now()) WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if found == 0 {
        return Err(ApiError::NotFound(
            "No staff member with this ID.".to_owned(),
        ));
    }
    let revoked = sqlx::query(
        "UPDATE sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    tracing::info!(%user_id, by = %staff.session.user_id, revoked, "staff member suspended");
    Ok(StatusCode::NO_CONTENT)
}

/// Lifts a suspension; the staff member can sign in again.
#[utoipa::path(
    delete,
    path = "/v1/staff/{user_id}/suspension",
    tag = "staff",
    security(("session" = [])),
    params(("user_id" = Uuid, Path, description = "The staff member's user ID")),
    responses(
        (status = NO_CONTENT, description = "Reinstated (also when not suspended)"),
        (status = UNAUTHORIZED, description = "`unauthorized`", body = ErrorBody),
        (status = FORBIDDEN, description = "`not_staff` or `forbidden` (needs `staff.suspend`)", body = ErrorBody),
        (status = NOT_FOUND, description = "`not_found`: no such staff member", body = ErrorBody),
    ),
)]
pub async fn reinstate(
    State(state): State<AppState>,
    staff: CurrentStaff,
    ApiPath(user_id): ApiPath<Uuid>,
) -> ApiResult<StatusCode> {
    staff.require(Permission::StaffSuspend)?;
    let found = sqlx::query("UPDATE staff_members SET suspended_at = NULL WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if found == 0 {
        return Err(ApiError::NotFound(
            "No staff member with this ID.".to_owned(),
        ));
    }
    tracing::info!(%user_id, by = %staff.session.user_id, "staff member reinstated");
    Ok(StatusCode::NO_CONTENT)
}
