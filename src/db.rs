use sqlx::SqlitePool;

pub async fn init_pool(database_url: &str) -> SqlitePool {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect(database_url)
        .await
        .unwrap()
}

pub async fn create_tables(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            code TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            hits INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}
