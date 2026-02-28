use sqlx::PgPool;
use tokio::sync::RwLock;

/// Shared application state accessible by all route handlers.
pub struct AppState {
    pub db: PgPool,
    /// Active bot instances (id → status).
    pub active_bots: RwLock<std::collections::HashMap<String, BotStatus>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BotStatus {
    pub id: String,
    pub strategy: String,
    pub instrument: String,
    pub status: String,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            active_bots: RwLock::new(std::collections::HashMap::new()),
        }
    }
}
