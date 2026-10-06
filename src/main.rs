mod codes;
mod db;
mod error;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get, post},
};

use handlers::{create_link, health, link_stats, redirect_link};
use models::AppState;

#[tokio::main]
async fn main() {
    let pool = db::init_pool().await;
    db::create_tables(&pool).await;

    let state = AppState { db_pool: pool };
    let app = Router::new()
        .route("/health", get(health))
        .route("/links", post(create_link))
        .route("/links/{id}/stats", get(link_stats))
        .route("/{id}", get(redirect_link))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
