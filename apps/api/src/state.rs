use sqlx::PgPool;

/// Shared by every handler. Cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
