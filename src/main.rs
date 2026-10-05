use axum::{
    Json, Router,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct CreateLink {
    url: String,
}

#[derive(Serialize)]
struct Link {
    id: String,
    url: String,
}

#[derive(Serialize)]
struct Stats {
    id: String,
    url: String,
    hits: i64,
    created_at: String,
}

async fn create_link(Json(payload): Json<CreateLink>) -> (StatusCode, Json<Link>) {
    (
        StatusCode::CREATED,
        Json(Link {
            id: "1".to_string(),
            url: payload.url,
        }),
    )
}

async fn redirect_link(Path(id): Path<String>) -> impl IntoResponse {
    let _ = id;
    (
        StatusCode::FOUND,
        [(header::LOCATION, "https://example.com")],
    )
}

async fn link_stats(Path(id): Path<String>) -> Json<Stats> {
    Json(Stats {
        id,
        url: "https://example.com".to_string(),
        hits: 0,
        created_at: "2026-10-05T00:00:00Z".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let _pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite:shortlink.db?mode=rwc")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            hits INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(&_pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/health", get(health))
        .route("/links", post(create_link))
        .route("/links/{id}/stats", get(link_stats))
        .route("/{id}", get(redirect_link));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
