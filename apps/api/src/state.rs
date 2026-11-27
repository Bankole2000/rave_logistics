use std::sync::Arc;

use sqlx::PgPool;

use crate::{auth::sender::MessageSender, config::AuthConfig};

/// Shared by every handler. Cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: AuthConfig,
    /// Delivers one-time codes and invitations by SMS or email.
    pub message_sender: Arc<dyn MessageSender>,
}
