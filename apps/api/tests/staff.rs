//! Staff invitations, roles and suspension (FT-072, FT-073), and sign-in by email for staff
//! (FT-099), against a real database (`docker compose up -d`).

mod common;

use api::auth::roles::Role;
use axum::http::{Method, StatusCode};
use common::{E164, TestApp};
use serde_json::{Value, json};
use sqlx::PgPool;

const INVITEE: &str = "0903 000 0000";
const INVITEE_E164: &str = "+2349030000000";

/// Settings that let the same number sign in several times in a row.
const QUICK: &[(&str, &str)] = &[("AUTH_CODE_RESEND_SECONDS", "0")];

async fn invite(app: &TestApp, token: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, body) = app
        .call(
            Method::POST,
            "/v1/staff/invitations",
            Some(token),
            Some(body),
        )
        .await;
    (status, body)
}

#[sqlx::test]
async fn an_invited_phone_number_becomes_staff_on_sign_in(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;

    let (status, invitation) =
        invite(&app, &admin, json!({ "phone": INVITEE, "role": "support" })).await;
    assert_eq!(status, StatusCode::CREATED, "{invitation}");
    assert_eq!(invitation["phone"], INVITEE_E164);
    assert_eq!(invitation["role"], "support");
    assert_eq!(
        app.sender.last_invitation(INVITEE_E164),
        Some(Role::Support)
    );

    let signed_in = app.sign_in(INVITEE).await;
    let staff = &signed_in["user"]["staff"];
    assert_eq!(staff["role"], "support");
    assert_eq!(staff["suspended"], false);
    let permissions = staff["permissions"].as_array().unwrap();
    assert!(permissions.contains(&json!("orders.search")));
    assert!(!permissions.contains(&json!("staff.create")));

    // Accepted invitations are no longer pending.
    let (_, _, pending) = app
        .call(Method::GET, "/v1/staff/invitations", Some(&admin), None)
        .await;
    assert_eq!(pending, json!([]));
}

#[sqlx::test]
async fn an_invited_email_address_can_sign_in_by_email(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    let (status, _) = invite(
        &app,
        &admin,
        json!({ "email": "Ada@Example.com", "role": "finance" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let signed_in = app.sign_in_email("ada@example.com").await;
    assert_eq!(signed_in["is_new_user"], true);
    assert_eq!(signed_in["user"]["email"], "ada@example.com");
    assert_eq!(signed_in["user"]["phone"], Value::Null);
    assert_eq!(signed_in["user"]["staff"]["role"], "finance");

    // Signing in by email again reaches the same user.
    let again = app.sign_in_email("ada@example.com").await;
    assert_eq!(again["is_new_user"], false);
    assert_eq!(again["user"]["id"], signed_in["user"]["id"]);
}

#[sqlx::test]
async fn an_email_without_an_account_or_invitation_gets_no_code(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (status, _, sent) = app
        .call(
            Method::POST,
            "/v1/auth/codes",
            None,
            Some(json!({ "email": "stranger@example.com" })),
        )
        .await;
    // Same answer as for a real account, but nothing is sent or stored.
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(sent["email"], "stranger@example.com");
    assert_eq!(app.sender.count(), 0);
    let (status, body) = app.verify(&sent["code_id"], "123456").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "code_expired");
}

#[sqlx::test]
async fn only_admins_can_invite_staff(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    let body = json!({ "phone": "0705 555 5555", "role": "admin" });

    let (status, _, error) = app
        .call(
            Method::POST,
            "/v1/staff/invitations",
            None,
            Some(body.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{error}");

    let customer = app.sign_in("0812 222 2222").await;
    let (status, error) = invite(&app, customer["token"].as_str().unwrap(), body.clone()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error["error"]["code"], "not_staff");

    invite(&app, &admin, json!({ "phone": INVITEE, "role": "support" })).await;
    let support = app.sign_in(INVITEE).await;
    let (status, error) = invite(&app, support["token"].as_str().unwrap(), body).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error["error"]["code"], "forbidden");
}

#[sqlx::test]
async fn hr_can_list_staff_but_not_invite(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    invite(&app, &admin, json!({ "phone": INVITEE, "role": "hr" })).await;
    let hr = app.sign_in(INVITEE).await;
    let hr = hr["token"].as_str().unwrap();

    let (status, _, staff) = app.call(Method::GET, "/v1/staff", Some(hr), None).await;
    assert_eq!(status, StatusCode::OK);
    let roles: Vec<&str> = staff
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["role"].as_str().unwrap())
        .collect();
    assert_eq!(roles, ["hr", "admin"]);

    let (status, _) = invite(&app, hr, json!({ "phone": "0705 555 5555", "role": "hr" })).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn inviting_a_staff_member_again_is_a_conflict(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    let (status, error) = invite(&app, &admin, json!({ "phone": E164, "role": "support" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "already_staff");
}

#[sqlx::test]
async fn a_new_invitation_replaces_the_pending_one(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    invite(&app, &admin, json!({ "phone": INVITEE, "role": "support" })).await;
    invite(&app, &admin, json!({ "phone": INVITEE, "role": "finance" })).await;

    let (_, _, pending) = app
        .call(Method::GET, "/v1/staff/invitations", Some(&admin), None)
        .await;
    let pending = pending.as_array().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["role"], "finance");

    let signed_in = app.sign_in(INVITEE).await;
    assert_eq!(signed_in["user"]["staff"]["role"], "finance");
}

#[sqlx::test]
async fn an_expired_invitation_is_not_accepted(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    invite(&app, &admin, json!({ "phone": INVITEE, "role": "support" })).await;
    sqlx::query("UPDATE staff_invitations SET expires_at = now() - interval '1 second'")
        .execute(&app.db)
        .await
        .unwrap();

    // The number can still sign in, as a customer, without staff access.
    let signed_in = app.sign_in(INVITEE).await;
    assert_eq!(signed_in["user"]["staff"], Value::Null);
}

#[sqlx::test]
async fn suspending_signs_out_everywhere_and_blocks_sign_in(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    invite(
        &app,
        &admin,
        json!({ "phone": INVITEE, "role": "operations" }),
    )
    .await;
    let first = app.sign_in(INVITEE).await;
    let second = app.sign_in(INVITEE).await;
    let user_id = first["user"]["id"].as_str().unwrap();
    let path = format!("/v1/staff/{user_id}/suspension");

    let (status, _, _) = app.call(Method::POST, &path, Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    for signed_in in [&first, &second] {
        let (status, _, _) = app.me(signed_in["token"].as_str().unwrap()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    let (_, _, sent) = app.request_code(INVITEE).await;
    let code = app.sender.last_code(INVITEE_E164).unwrap();
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "account_suspended");

    let (_, _, staff) = app.call(Method::GET, "/v1/staff", Some(&admin), None).await;
    let member = staff
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == user_id)
        .unwrap();
    assert!(member["suspended_at"].is_string());

    // Reinstated: they can sign in again with their role.
    let (status, _, _) = app.call(Method::DELETE, &path, Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let signed_in = app.sign_in(INVITEE).await;
    assert_eq!(signed_in["user"]["staff"]["role"], "operations");
}

#[sqlx::test]
async fn suspension_errors(db: PgPool) {
    let app = TestApp::new(db, QUICK);
    let admin = app.admin(E164).await;
    let (_, _, me) = app.me(&admin).await;
    let own = format!("/v1/staff/{}/suspension", me["id"].as_str().unwrap());

    let (status, _, body) = app.call(Method::POST, &own, Some(&admin), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "cannot_suspend_self");

    let unknown = format!("/v1/staff/{}/suspension", uuid::Uuid::new_v4());
    let (status, _, body) = app.call(Method::POST, &unknown, Some(&admin), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    let (status, _, body) = app
        .call(
            Method::POST,
            "/v1/staff/not-a-uuid/suspension",
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_path");
}
