use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::handlers;

pub fn build_router(pool: PgPool) -> Router {
    Router::new()
        // Health (no auth)
        .route("/health", get(handlers::health::health))
        // Events
        .route("/events", post(handlers::events::create_event))
        .route("/events", get(handlers::events::list_events))
        .route("/events/{id}", get(handlers::events::get_event))
        .route("/events/{id}", delete(handlers::events::delete_event))
        // API keys
        .route("/api-keys", post(handlers::api_keys::create_api_key))
        .route("/api-keys", get(handlers::api_keys::list_api_keys))
        .route("/api-keys/{id}", delete(handlers::api_keys::revoke_api_key))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool)
}
