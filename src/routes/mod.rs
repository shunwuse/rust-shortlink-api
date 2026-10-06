pub mod health;
pub mod links;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use crate::models::AppState;

use health::health;
use links::{create_link, link_stats, redirect_link};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/links", post(create_link))
        .route("/links/{id}/stats", get(link_stats))
        .route("/{id}", get(redirect_link))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
