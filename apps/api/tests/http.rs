//! HTTP-level tests against the full app with all middleware.
//! Tests marked `#[sqlx::test]` need the local database (`docker compose up -d`) and DATABASE_URL;
//! each gets its own temporary database with the migrations applied.

use api::{config::Config, state::AppState};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

fn test_config() -> Config {
    Config::from_lookup(|key| match key {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "API_CORS_ORIGINS" => Some("http://127.0.0.1:13001".to_owned()),
        _ => None,
    })
    .unwrap()
}

fn app(db: PgPool) -> Router {
    api::app(AppState { db }, &test_config())
}

/// A pool pointing at a port where nothing listens, to simulate a database outage.
fn unreachable_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect_lazy("postgres://postgres@127.0.0.1:1/none")
        .unwrap()
}

async fn send(app: Router, req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Value) {
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, body)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

#[sqlx::test]
async fn health_is_ok_when_the_database_answers(db: PgPool) {
    let (status, _, body) = send(app(db), get("/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[sqlx::test]
async fn postgis_is_installed_by_the_migrations(db: PgPool) {
    let (version,): (String,) = sqlx::query_as("SELECT postgis_version()")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(!version.is_empty());
}

#[tokio::test]
async fn health_is_unavailable_when_the_database_is_down() {
    let (status, _, body) = send(app(unreachable_pool()), get("/health")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "unavailable");
    assert_eq!(body["database"], "unavailable");
}

#[tokio::test]
async fn unknown_routes_return_the_standard_error_body() {
    let (status, _, body) = send(app(unreachable_pool()), get("/nope")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["message"], "No route for GET /nope");
}

#[tokio::test]
async fn every_response_has_a_request_id() {
    let (_, headers, _) = send(app(unreachable_pool()), get("/nope")).await;
    let id = headers.get("x-request-id").expect("x-request-id header");
    assert!(uuid::Uuid::parse_str(id.to_str().unwrap()).is_ok());
}

#[tokio::test]
async fn a_client_request_id_is_kept() {
    let req = Request::get("/nope")
        .header("x-request-id", "client-abc-123")
        .body(Body::empty())
        .unwrap();
    let (_, headers, _) = send(app(unreachable_pool()), req).await;
    assert_eq!(headers["x-request-id"], "client-abc-123");
}

#[tokio::test]
async fn cors_allows_configured_origins_only() {
    let preflight = |origin: &str| {
        Request::builder()
            .method("OPTIONS")
            .uri("/health")
            .header(header::ORIGIN, origin)
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .body(Body::empty())
            .unwrap()
    };

    let (_, headers, _) = send(app(unreachable_pool()), preflight("http://127.0.0.1:13001")).await;
    assert_eq!(
        headers[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://127.0.0.1:13001"
    );

    let (_, headers, _) = send(app(unreachable_pool()), preflight("https://evil.test")).await;
    assert!(headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
}

#[test]
fn openapi_describes_health() {
    let doc = api::openapi();
    assert!(doc.paths.paths.contains_key("/health"));
    assert_eq!(doc.info.version, env!("CARGO_PKG_VERSION"));
}
