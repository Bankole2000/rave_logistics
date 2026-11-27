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

/// Where one-time codes and staff invitations go. Only `console` exists until the SMS and email
/// providers are chosen (PRD, open decisions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeSenderKind {
    /// Prints messages to api's standard error. Local development only; refused in release builds.
    Console,
}

impl FromStr for CodeSenderKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "console" => Ok(Self::Console),
            other => bail!("AUTH_CODE_SENDER must be console, got {other:?}"),
        }
    }
}

/// Sign-in limits. Each is a setting with a default (AGENTS.md, configurable TBDs).
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub code_sender: CodeSenderKind,
    /// How long a one-time code stays valid.
    pub code_ttl_secs: i64,
    /// Wrong guesses allowed per code before it stops working.
    pub code_max_attempts: i32,
    /// Minimum wait before another code can be sent to the same phone number.
    pub code_resend_secs: i64,
    /// Codes allowed per phone number in any rolling hour.
    pub code_max_per_hour: i64,
    /// A session unused for this long is signed out.
    pub session_idle_days: i32,
    /// How long a staff invitation can be accepted.
    pub invitation_days: i32,
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
    pub auth: AuthConfig,
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
            database_url: get("DATABASE_URL").context(
                "DATABASE_URL is required. For local use: cp apps/api/.env.example apps/api/.env",
            )?,
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
            auth: AuthConfig {
                code_sender: or("AUTH_CODE_SENDER", "console").parse()?,
                code_ttl_secs: number(&get, "AUTH_CODE_TTL_SECONDS", 300)?,
                code_max_attempts: number(&get, "AUTH_CODE_MAX_ATTEMPTS", 5)?,
                code_resend_secs: number(&get, "AUTH_CODE_RESEND_SECONDS", 60)?,
                code_max_per_hour: number(&get, "AUTH_CODE_MAX_PER_HOUR", 5)?,
                session_idle_days: number(&get, "AUTH_SESSION_IDLE_DAYS", 90)?,
                invitation_days: number(&get, "STAFF_INVITATION_DAYS", 7)?,
            },
        })
    }
}

/// Reads a non-negative number setting, or its default when unset.
fn number<T>(get: &impl Fn(&str) -> Option<String>, key: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr + PartialOrd + Default,
{
    let Some(raw) = get(key) else {
        return Ok(default);
    };
    match raw.trim().parse::<T>() {
        Ok(v) if v >= T::default() => Ok(v),
        _ => bail!("{key} must be a non-negative whole number, got {raw:?}"),
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
    fn auth_defaults_apply() {
        let config = Config::from_lookup(lookup(&[("DATABASE_URL", "postgres://x")])).unwrap();
        assert_eq!(config.auth.code_sender, CodeSenderKind::Console);
        assert_eq!(config.auth.code_ttl_secs, 300);
        assert_eq!(config.auth.code_max_attempts, 5);
        assert_eq!(config.auth.code_resend_secs, 60);
        assert_eq!(config.auth.code_max_per_hour, 5);
        assert_eq!(config.auth.session_idle_days, 90);
        assert_eq!(config.auth.invitation_days, 7);
    }

    #[test]
    fn negative_or_non_numeric_auth_settings_are_rejected() {
        for value in ["-1", "five"] {
            let result = Config::from_lookup(lookup(&[
                ("DATABASE_URL", "postgres://x"),
                ("AUTH_CODE_MAX_ATTEMPTS", value),
            ]));
            assert!(result.is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn unknown_code_sender_is_rejected() {
        let result = Config::from_lookup(lookup(&[
            ("DATABASE_URL", "postgres://x"),
            ("AUTH_CODE_SENDER", "carrier-pigeon"),
        ]));
        assert!(result.is_err());
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
