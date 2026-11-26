//! Configuration from environment variables. Every variable is documented in `.env.example`.

use std::{net::SocketAddr, str::FromStr};

use anyhow::{Context, bail};

/// Default allowed client origins: the local web dev servers and the Tauri shells.
pub const DEFAULT_CORS_ORIGINS: &str = "http://127.0.0.1:13001,http://127.0.0.1:13002,http://127.0.0.1:13003,http://127.0.0.1:13004,tauri://localhost,http://tauri.localhost,https://tauri.localhost";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl FromStr for LogFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            other => bail!("API_LOG_FORMAT must be pretty or json, got {other:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub db_max_connections: u32,
    pub db_acquire_timeout_secs: u64,
    pub cors_origins: Vec<String>,
    pub log_format: LogFormat,
    pub migrate_on_start: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// Builds the configuration from any key lookup, so tests don't touch the process environment.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> anyhow::Result<Self> {
        let or = |key: &str, default: &str| get(key).unwrap_or_else(|| default.to_owned());

        Ok(Self {
            bind_addr: or("API_BIND_ADDR", "127.0.0.1:18082")
                .parse()
                .context("API_BIND_ADDR must be host:port")?,
            database_url: get("DATABASE_URL").context("DATABASE_URL is required")?,
            db_max_connections: or("API_DB_MAX_CONNECTIONS", "10")
                .parse()
                .context("API_DB_MAX_CONNECTIONS must be a number")?,
            db_acquire_timeout_secs: or("API_DB_ACQUIRE_TIMEOUT_SECONDS", "3")
                .parse()
                .context("API_DB_ACQUIRE_TIMEOUT_SECONDS must be a number")?,
            cors_origins: parse_list(&or("API_CORS_ORIGINS", DEFAULT_CORS_ORIGINS)),
            log_format: or("API_LOG_FORMAT", "pretty").parse()?,
            migrate_on_start: or("API_MIGRATE_ON_START", "true")
                .parse()
                .context("API_MIGRATE_ON_START must be true or false")?,
        })
    }
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn defaults_apply_when_only_database_url_is_set() {
        let config = Config::from_lookup(lookup(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(config.bind_addr.to_string(), "127.0.0.1:18082");
        assert_eq!(config.db_max_connections, 10);
        assert_eq!(config.log_format, LogFormat::Pretty);
        assert!(config.migrate_on_start);
        assert!(
            config
                .cors_origins
                .contains(&"tauri://localhost".to_owned())
        );
    }

    #[test]
    fn missing_database_url_is_an_error() {
        let err = Config::from_lookup(lookup(&[])).unwrap_err();
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn cors_origins_are_trimmed_and_blank_entries_dropped() {
        let config = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("API_CORS_ORIGINS", " http://a.test , ,http://b.test"),
        ]))
        .unwrap();
        assert_eq!(config.cors_origins, ["http://a.test", "http://b.test"]);
    }

    #[test]
    fn invalid_log_format_is_rejected() {
        let result = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("API_LOG_FORMAT", "xml"),
        ]));
        assert!(result.is_err());
    }
}
