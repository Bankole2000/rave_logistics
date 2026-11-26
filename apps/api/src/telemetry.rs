use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::LogFormat;

/// Logs go to stdout. The level comes from `RUST_LOG` (default `info`).
/// Never log secrets, one-time codes, tokens or personal data (TECHNICAL.md, Security and privacy).
pub fn init(format: LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry().with(filter);
    match format {
        LogFormat::Json => registry.with(fmt::layer().json()).init(),
        LogFormat::Pretty => registry.with(fmt::layer()).init(),
    }
}
