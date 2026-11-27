//! Sign-in with a phone number and a one-time code, sessions and sign-out, against a real
//! database (`docker compose up -d`). Codes go to an in-memory sender instead of SMS.

use std::sync::Arc;

use api::{auth::sender::MemorySender, config::Config, state::AppState};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const PHONE: &str = "0803 123 4567";
const E164: &str = "+2348031234567";

struct TestApp {
    router: Router,
    sender: Arc<MemorySender>,
    db: PgPool,
}

impl TestApp {
    /// The app with default settings, except those in `overrides`.
    fn new(db: PgPool, overrides: &[(&str, &str)]) -> Self {
        let config = Config::from_lookup(|key| match key {
            "DATABASE_URL" => Some("postgres://unused".to_owned()),
            _ => overrides
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned()),
        })
        .unwrap();
        let sender = Arc::new(MemorySender::default());
        let state = AppState {
            db: db.clone(),
            auth: config.auth.clone(),
            code_sender: sender.clone(),
        };
        Self {
            router: api::app(state, &config),
            sender,
            db,
        }
    }

    async fn call(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, HeaderMap, Value) {
        let mut req = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            req = req.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let req = match body {
            Some(body) => req
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.router.clone().oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, headers, body)
    }

    async fn request_code(&self, phone: &str) -> (StatusCode, HeaderMap, Value) {
        self.call(
            Method::POST,
            "/v1/auth/codes",
            None,
            Some(json!({ "phone": phone })),
        )
        .await
    }

    async fn verify(&self, code_id: &Value, code: &str) -> (StatusCode, Value) {
        let (status, _, body) = self
            .call(
                Method::POST,
                "/v1/auth/sessions",
                None,
                Some(json!({ "code_id": code_id, "code": code, "device_name": "Test phone" })),
            )
            .await;
        (status, body)
    }

    /// Requests a code and enters it; returns the sign-in response.
    async fn sign_in(&self, phone: &str) -> Value {
        let (status, _, sent) = self.request_code(phone).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{sent}");
        let code = self
            .sender
            .last_code(sent["phone"].as_str().unwrap())
            .unwrap();
        let (status, body) = self.verify(&sent["code_id"], &code).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body
    }

    async fn me(&self, token: &str) -> (StatusCode, HeaderMap, Value) {
        self.call(Method::GET, "/v1/me", Some(token), None).await
    }
}

/// A code that differs from `code`.
fn wrong(code: &str) -> String {
    if code == "000000" { "000001" } else { "000000" }.to_owned()
}

#[sqlx::test]
async fn a_new_number_gets_an_account_after_entering_the_code(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (status, _, sent) = app.request_code(PHONE).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(sent["phone"], E164);
    assert_eq!(sent["expires_in_seconds"], 300);
    assert_eq!(sent["resend_after_seconds"], 60);

    let code = app.sender.last_code(E164).expect("a code was sent");
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["is_new_user"], true);
    assert_eq!(body["user"]["phone"], E164);
    assert!(body["token"].as_str().unwrap().starts_with("rcs_"));
}

#[sqlx::test]
async fn signing_in_again_uses_the_same_account(db: PgPool) {
    let app = TestApp::new(db, &[("AUTH_CODE_RESEND_SECONDS", "0")]);
    let first = app.sign_in(PHONE).await;
    // A different way of writing the same number.
    let second = app.sign_in("+2348031234567").await;
    assert_eq!(second["is_new_user"], false);
    assert_eq!(second["user"]["id"], first["user"]["id"]);
    assert_ne!(second["token"], first["token"]);
}

#[sqlx::test]
async fn a_session_token_gives_access_to_me(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let signed_in = app.sign_in(PHONE).await;
    let (status, _, me) = app.me(signed_in["token"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["id"], signed_in["user"]["id"]);
    assert_eq!(me["phone"], E164);
}

#[sqlx::test]
async fn me_without_a_valid_token_is_unauthorized(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let fake = format!("rcs_{}", "A".repeat(43));
    for token in [None, Some("not-a-token"), Some(fake.as_str())] {
        let (status, headers, body) = app.call(Method::GET, "/v1/me", token, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{token:?}");
        assert_eq!(body["error"]["code"], "unauthorized");
        assert_eq!(headers[header::WWW_AUTHENTICATE], "Bearer");
    }
}

#[sqlx::test]
async fn signing_out_ends_only_that_session(db: PgPool) {
    let app = TestApp::new(db, &[("AUTH_CODE_RESEND_SECONDS", "0")]);
    let phone_a = app.sign_in(PHONE).await;
    let phone_b = app.sign_in(PHONE).await;
    let (token_a, token_b) = (
        phone_a["token"].as_str().unwrap(),
        phone_b["token"].as_str().unwrap(),
    );

    let (status, _, _) = app
        .call(
            Method::DELETE,
            "/v1/auth/sessions/current",
            Some(token_a),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(app.me(token_a).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(app.me(token_b).await.0, StatusCode::OK);

    let (status, _, _) = app
        .call(
            Method::DELETE,
            "/v1/auth/sessions/current",
            Some(token_a),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn an_invalid_phone_number_is_rejected(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (status, _, body) = app.request_code("12345").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_phone");
    assert_eq!(app.sender.count(), 0);
}

#[sqlx::test]
async fn a_malformed_body_gets_the_standard_error_body(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (status, _, body) = app
        .call(
            Method::POST,
            "/v1/auth/codes",
            None,
            Some(json!({ "number": PHONE })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_body");
}

#[sqlx::test]
async fn a_second_code_within_the_resend_wait_is_refused(db: PgPool) {
    let app = TestApp::new(db, &[]);
    assert_eq!(app.request_code(PHONE).await.0, StatusCode::ACCEPTED);
    let (status, headers, body) = app.request_code(PHONE).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "too_many_requests");
    let retry: i64 = headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=60).contains(&retry), "{retry}");
    assert_eq!(app.sender.count(), 1);
}

#[sqlx::test]
async fn codes_per_hour_are_limited(db: PgPool) {
    let app = TestApp::new(
        db,
        &[
            ("AUTH_CODE_RESEND_SECONDS", "0"),
            ("AUTH_CODE_MAX_PER_HOUR", "2"),
        ],
    );
    assert_eq!(app.request_code(PHONE).await.0, StatusCode::ACCEPTED);
    assert_eq!(app.request_code(PHONE).await.0, StatusCode::ACCEPTED);
    let (status, headers, _) = app.request_code(PHONE).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    let retry: i64 = headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((3500..=3600).contains(&retry), "{retry}");
    // Other numbers aren't affected.
    assert_eq!(
        app.request_code("0903 000 0000").await.0,
        StatusCode::ACCEPTED
    );
}

#[sqlx::test]
async fn a_wrong_code_is_rejected_and_attempts_are_limited(db: PgPool) {
    let app = TestApp::new(db, &[("AUTH_CODE_MAX_ATTEMPTS", "2")]);
    let (_, _, sent) = app.request_code(PHONE).await;
    let code = app.sender.last_code(E164).unwrap();

    for _ in 0..2 {
        let (status, body) = app.verify(&sent["code_id"], &wrong(&code)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "invalid_code");
    }
    // Out of attempts: even the right code no longer works.
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "code_expired");
}

#[sqlx::test]
async fn a_code_works_only_once(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (_, _, sent) = app.request_code(PHONE).await;
    let code = app.sender.last_code(E164).unwrap();
    assert_eq!(app.verify(&sent["code_id"], &code).await.0, StatusCode::OK);
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "code_expired");
}

#[sqlx::test]
async fn an_expired_or_unknown_code_is_rejected(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let (_, _, sent) = app.request_code(PHONE).await;
    let code = app.sender.last_code(E164).unwrap();
    sqlx::query("UPDATE sign_in_codes SET expires_at = now() - interval '1 second'")
        .execute(&app.db)
        .await
        .unwrap();
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "code_expired");

    let unknown = json!(uuid::Uuid::new_v4());
    let (status, body) = app.verify(&unknown, "123456").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "code_expired");
}

#[sqlx::test]
async fn a_suspended_user_cant_sign_in_and_their_sessions_stop_working(db: PgPool) {
    let app = TestApp::new(db, &[("AUTH_CODE_RESEND_SECONDS", "0")]);
    let signed_in = app.sign_in(PHONE).await;
    sqlx::query("UPDATE users SET suspended_at = now()")
        .execute(&app.db)
        .await
        .unwrap();

    assert_eq!(
        app.me(signed_in["token"].as_str().unwrap()).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (_, _, sent) = app.request_code(PHONE).await;
    let code = app.sender.last_code(E164).unwrap();
    let (status, body) = app.verify(&sent["code_id"], &code).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "account_suspended");
}

#[sqlx::test]
async fn an_unused_session_expires_after_the_idle_period(db: PgPool) {
    let app = TestApp::new(db, &[("AUTH_SESSION_IDLE_DAYS", "30")]);
    let token = app.sign_in(PHONE).await["token"]
        .as_str()
        .unwrap()
        .to_owned();
    sqlx::query("UPDATE sessions SET last_seen_at = now() - interval '29 days'")
        .execute(&app.db)
        .await
        .unwrap();
    // Using it within the period keeps it alive...
    assert_eq!(app.me(&token).await.0, StatusCode::OK);
    sqlx::query("UPDATE sessions SET last_seen_at = now() - interval '31 days'")
        .execute(&app.db)
        .await
        .unwrap();
    // ...and after the period it no longer works.
    assert_eq!(app.me(&token).await.0, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn only_hashes_of_codes_and_tokens_are_stored(db: PgPool) {
    let app = TestApp::new(db, &[]);
    let signed_in = app.sign_in(PHONE).await;
    let token = signed_in["token"].as_str().unwrap();
    let code = app.sender.last_code(E164).unwrap();

    let (dump,): (String,) = sqlx::query_as(
        "SELECT (SELECT string_agg(to_jsonb(s)::text, '') FROM sessions s)
             || (SELECT string_agg(to_jsonb(c)::text, '') FROM sign_in_codes c)",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert!(!dump.contains(token));
    assert!(!dump.contains(&format!("\"{code}\"")));
    let (device,): (Option<String>,) = sqlx::query_as("SELECT device_name FROM sessions")
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(device.as_deref(), Some("Test phone"));
}
