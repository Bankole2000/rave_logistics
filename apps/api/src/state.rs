use std::sync::Arc;

use sqlx::PgPool;

use crate::{auth::sender::CodeSender, config::AuthConfig};

/// Shared by every handler. Cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub auth: AuthConfig,
    pub code_sender: Arc<dyn CodeSender>,
}
