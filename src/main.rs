mod auth;
mod config;
mod db;
mod handlers;
mod models;
mod response;
mod routes;

use sha2::{Digest, Sha256};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Load .env if present (ignore error if missing)
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = config::Config::from_env();
    tracing::info!("Starting on port {}", config.port);

    let pool = db::create_pool(&config.database_url).await;
    db::run_migrations(&pool).await;

    // Seed API key if configured
    if let Some(ref seed_key) = config.api_key_seed {
        let mut hasher = Sha256::new();
        hasher.update(seed_key.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM api_keys WHERE key_hash = $1)")
                .bind(&hash)
                .fetch_one(&pool)
                .await
                .unwrap_or(false);

        if !exists {
            sqlx::query("INSERT INTO api_keys (name, key_hash, scopes) VALUES ($1, $2, $3)")
                .bind("seed-admin")
                .bind(&hash)
                .bind(vec!["*"])
                .execute(&pool)
                .await
                .expect("Failed to seed API key");

            tracing::info!("Seeded admin API key (plaintext in API_KEY_SEED env var)");
            tracing::info!("Plaintext key: {seed_key}");
        }
    }

    let app = routes::build_router(pool);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Listening on {addr}");
    axum::serve(listener, app).await.expect("Server error");
}
