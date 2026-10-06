use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct CreateLink {
    pub url: String,
}

#[derive(Serialize)]
pub struct Link {
    pub id: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct ErrorBody {
    pub error: String,
}

#[derive(Serialize)]
pub struct Stats {
    pub id: String,
    pub url: String,
    pub hits: i64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
}
