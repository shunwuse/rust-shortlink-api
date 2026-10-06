mod codes;
mod config;
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
    let config = config::Config::from_env();

    let pool = db::init_pool(&config.database_url).await;
    db::create_tables(&pool).await;

    let state = AppState { db_pool: pool };
    let app = Router::new()
        .route("/health", get(health))
        .route("/links", post(create_link))
        .route("/links/{id}/stats", get(link_stats))
        .route("/{id}", get(redirect_link))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
