mod codes;
mod config;
mod db;
mod error;
mod models;
mod routes;

use models::AppState;

#[tokio::main]
async fn main() {
    let config = config::Config::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(config.log_filter.as_str())
        .init();

    let pool = db::init_pool(&config.database_url).await;
    db::create_tables(&pool).await;

    let state = AppState { db_pool: pool };
    let app = routes::create_router(state);

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
