//! Test helpers shared by the HTTP test files: the whole app with an in-memory message sender.
#![allow(dead_code)]

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

pub const PHONE: &str = "0803 123 4567";
pub const E164: &str = "+2348031234567";

pub struct TestApp {
    pub router: Router,
    pub sender: Arc<MemorySender>,
    pub db: PgPool,
}

impl TestApp {
    /// The app with default settings, except those in `overrides`.
    pub fn new(db: PgPool, overrides: &[(&str, &str)]) -> Self {
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
            message_sender: sender.clone(),
        };
        Self {
            router: api::app(state, &config),
            sender,
            db,
        }
    }

    pub async fn call(
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

    pub async fn request_code(&self, phone: &str) -> (StatusCode, HeaderMap, Value) {
        self.call(
            Method::POST,
            "/v1/auth/codes",
            None,
            Some(json!({ "phone": phone })),
        )
        .await
    }

    pub async fn verify(&self, code_id: &Value, code: &str) -> (StatusCode, Value) {
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
    pub async fn sign_in(&self, phone: &str) -> Value {
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

    pub async fn me(&self, token: &str) -> (StatusCode, HeaderMap, Value) {
        self.call(Method::GET, "/v1/me", Some(token), None).await
    }
}

impl TestApp {
    /// Signs in by email (the address must have a user or a pending invitation).
    pub async fn sign_in_email(&self, email: &str) -> Value {
        let (status, _, sent) = self
            .call(
                Method::POST,
                "/v1/auth/codes",
                None,
                Some(json!({ "email": email })),
            )
            .await;
        assert_eq!(status, StatusCode::ACCEPTED, "{sent}");
        let code = self
            .sender
            .last_code(email)
            .expect("an email code was sent");
        let (status, body) = self.verify(&sent["code_id"], &code).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body
    }

    /// Makes the user with this phone number (E.164) an admin directly in the database and
    /// returns a session token for them.
    pub async fn admin(&self, phone: &str) -> String {
        let signed_in = self.sign_in(phone).await;
        let user_id: uuid::Uuid = signed_in["user"]["id"].as_str().unwrap().parse().unwrap();
        sqlx::query("INSERT INTO staff_members (user_id, role) VALUES ($1, 'admin')")
            .bind(user_id)
            .execute(&self.db)
            .await
            .unwrap();
        signed_in["token"].as_str().unwrap().to_owned()
    }
}
