//! Rave Connect Logistics api: the only owner of the database. Every client and server calls it.

pub mod config;
pub mod error;
pub mod routes;
pub mod state;
pub mod telemetry;

use std::time::Duration;

use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, Request, header},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{config::Config, error::ApiError, state::AppState};

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rave Connect Logistics api",
        description = "The api used by every Rave Connect Logistics client. Errors use the ErrorBody schema; each response carries an `x-request-id` header.",
        license(name = "MIT", identifier = "MIT"),
    ),
    components(schemas(error::ErrorBody, error::ErrorDetail)),
    tags((name = "health", description = "Service health")),
)]
struct ApiDoc;

/// The routes and their OpenAPI description, collected together so they can't drift apart.
fn api_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(routes::health::health))
}

/// The OpenAPI document for api (written to packages/api-contract by the `openapi` binary).
pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, mut doc) = api_router().split_for_parts();
    doc.info.version = env!("CARGO_PKG_VERSION").to_owned();
    doc
}

/// Builds the HTTP application with all middleware.
pub fn app(state: AppState, config: &Config) -> Router {
    let (router, _) = api_router().split_for_parts();

    // Outermost layer last: a request ID is set first, so the trace span and the response carry it.
    router
        .fallback(fallback)
        .with_state(state)
        .layer(cors(&config.cors_origins))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                let request_id = req
                    .headers()
                    .get(REQUEST_ID_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default();
                tracing::info_span!("request", method = %req.method(), path = %req.uri().path(), request_id)
            }),
        )
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
}

fn cors(origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| match HeaderValue::from_str(o) {
            Ok(v) => Some(v),
            Err(_) => {
                tracing::warn!(origin = %o, "ignoring invalid CORS origin");
                None
            }
        })
        .collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            REQUEST_ID_HEADER,
        ])
        .expose_headers([REQUEST_ID_HEADER])
        .max_age(Duration::from_secs(600))
}

async fn fallback(method: Method, uri: axum::http::Uri) -> ApiError {
    ApiError::NotFound(format!("No route for {method} {}", uri.path()))
}

/// A pool that connects on first use, so api starts (and reports itself unhealthy) while the database is down.
pub fn connect_pool(config: &Config) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .acquire_timeout(Duration::from_secs(config.db_acquire_timeout_secs))
        .connect_lazy(&config.database_url)?)
}

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
