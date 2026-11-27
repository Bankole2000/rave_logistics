//! Creates an admin invitation without an existing admin: the way to set up the first admin of an
//! environment. The invited person accepts by signing in with that phone number or email.
//!
//! ```text
//! cargo run -p api --bin invite-admin -- --phone 08031234567
//! cargo run -p api --bin invite-admin -- --email ada@example.com
//! ```
//!
//! Uses DATABASE_URL (and apps/api/.env locally) and applies pending migrations first.

use anyhow::{Context, bail};
use api::{
    auth::{destination::Destination, roles::Role},
    config::Config,
    routes::staff::create_invitation,
};

const USAGE: &str = "usage: invite-admin (--phone <number> | --email <address>)";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    #[cfg(debug_assertions)]
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")).ok();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let to = match args.as_slice() {
        [flag, value] if flag == "--phone" => Destination::from_request(Some(value), None),
        [flag, value] if flag == "--email" => Destination::from_request(None, Some(value)),
        _ => bail!(USAGE),
    }
    .map_err(|err| anyhow::anyhow!("{err}"))?;

    let config = Config::from_env()?;
    let db = api::connect_pool(&config)?;
    api::MIGRATOR
        .run(&db)
        .await
        .context("running database migrations (is the database up? `docker compose up -d`)")?;

    let mut tx = db.begin().await?;
    let invitation =
        create_invitation(&mut tx, &to, Role::Admin, None, config.auth.invitation_days)
            .await
            .map_err(|err| anyhow::anyhow!("{err}"))?;
    tx.commit().await?;

    println!(
        "Admin invitation created for {} (expires {}). Sign in with it to accept.",
        to.masked(),
        invitation.expires_at.format("%Y-%m-%d %H:%M UTC"),
    );
    Ok(())
}
