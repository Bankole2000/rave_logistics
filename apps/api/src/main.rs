use anyhow::Context;
use api::{config::Config, state::AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    let config = Config::from_env()?;
    api::telemetry::init(config.log_format);

    let db = api::connect_pool(&config)?;
    if config.migrate_on_start {
        api::MIGRATOR
            .run(&db)
            .await
            .context("running database migrations (is the database up? `docker compose up -d`)")?;
    }

    let app = api::app(AppState { db }, &config);
    let listener = TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("binding {}", config.bind_addr))?;
    tracing::info!(addr = %config.bind_addr, "api listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Loads a local `.env`; real environments set variables directly. Variables already set win,
/// and a missing file is ignored. Looks in the current directory, then (in debug builds only)
/// in `apps/api/`, so `cargo run -p api` works from the repo root as well as from the app folder.
fn load_dotenv() {
    dotenvy::dotenv().ok();
    #[cfg(debug_assertions)]
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")).ok();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
