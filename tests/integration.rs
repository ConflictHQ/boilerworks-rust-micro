use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// Spin up a real server on a random port and return (base_url, pool).
async fn spawn_app() -> (String, PgPool) {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5439/boilerworks".into());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Clean tables for test isolation
    sqlx::query("DELETE FROM events")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM api_keys")
        .execute(&pool)
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");

    let app = boilerworks_rust_micro::build_router(pool.clone());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (base_url, pool)
}

/// Insert a test API key directly into the DB and return its plaintext.
async fn seed_key(pool: &PgPool, name: &str, scopes: &[&str]) -> String {
    let plaintext = format!("test_key_{}", uuid::Uuid::new_v4());
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let hash = hex::encode(hasher.finalize());
    let scopes_vec: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();

    sqlx::query("INSERT INTO api_keys (name, key_hash, scopes) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(&hash)
        .bind(&scopes_vec)
        .execute(pool)
        .await
        .expect("Failed to seed test key");

    plaintext
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_returns_ok() {
    let (base, _pool) = spawn_app().await;
    let resp = reqwest::get(format!("{base}/health")).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"], "ok");
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_missing_key_returns_401() {
    let (base, _pool) = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("{base}/events")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_invalid_key_returns_401() {
    let (base, _pool) = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", "totally-bogus-key")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_valid_key_returns_200() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "test-valid", &["events.read"]).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Scopes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn scope_read_only_cannot_write() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "read-only", &["events.read"]).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn scope_wildcard_grants_all() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "admin", &["*"]).await;
    let client = reqwest::Client::new();

    // Can write events
    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Can read events
    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Can manage keys
    let resp = client
        .get(format!("{base}/api-keys"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Events CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn events_create() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "writer", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "order.created", "payload": {"amount": 100}}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["type"], "order.created");
    assert_eq!(body["data"]["payload"]["amount"], 100);
}

#[tokio::test]
async fn events_list() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "rw", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();

    // Create two events
    for t in &["a.test", "b.test"] {
        client
            .post(format!("{base}/events"))
            .header("X-API-Key", &key)
            .json(&serde_json::json!({"type": t}))
            .send()
            .await
            .unwrap();
    }

    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
}

#[tokio::test]
async fn events_list_filter_by_type() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "rw", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();

    for t in &["order.created", "order.created", "user.signup"] {
        client
            .post(format!("{base}/events"))
            .header("X-API-Key", &key)
            .json(&serde_json::json!({"type": t}))
            .send()
            .await
            .unwrap();
    }

    let resp = client
        .get(format!("{base}/events?type=order.created"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
}

#[tokio::test]
async fn events_get_by_id() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "rw", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "fetch.test"}))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap();

    let resp = client
        .get(format!("{base}/events/{id}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["type"], "fetch.test");
}

#[tokio::test]
async fn events_not_found_returns_404() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "reader", &["events.read"]).await;
    let client = reqwest::Client::new();
    let fake_id = uuid::Uuid::new_v4();
    let resp = client
        .get(format!("{base}/events/{fake_id}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn events_soft_delete() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "rw", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();

    // Create
    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "delete.test"}))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap();

    // Delete
    let resp = client
        .delete(format!("{base}/events/{id}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // GET returns 404
    let resp = client
        .get(format!("{base}/events/{id}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn events_deleted_not_in_list() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "rw", &["events.write", "events.read"]).await;
    let client = reqwest::Client::new();

    // Create two events
    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "keep"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = client
        .post(format!("{base}/events"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"type": "remove"}))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap();

    // Delete second
    client
        .delete(format!("{base}/events/{id}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();

    // List should only have one
    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["type"], "keep");
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_keys_create() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "admin", &["*"]).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/api-keys"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"name": "new-key", "scopes": ["events.read"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert!(body["data"]["plaintext_key"]
        .as_str()
        .unwrap()
        .starts_with("bw_"));
}

#[tokio::test]
async fn api_keys_list() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "admin", &["*"]).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api-keys"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    // At least the seeded admin key
    assert!(!body["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn api_keys_revoke() {
    let (base, pool) = spawn_app().await;
    let admin_key = seed_key(&pool, "admin", &["*"]).await;
    let client = reqwest::Client::new();

    // Create a new key to revoke
    let resp = client
        .post(format!("{base}/api-keys"))
        .header("X-API-Key", &admin_key)
        .json(&serde_json::json!({"name": "to-revoke", "scopes": ["events.read"]}))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["data"]["id"].as_str().unwrap();

    // Revoke it
    let resp = client
        .delete(format!("{base}/api-keys/{id}"))
        .header("X-API-Key", &admin_key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the revoked key no longer works
    let revoked_plaintext = body["data"]["plaintext_key"].as_str().unwrap();
    let resp = client
        .get(format!("{base}/events"))
        .header("X-API-Key", revoked_plaintext)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn api_keys_missing_name_validation() {
    let (base, pool) = spawn_app().await;
    let key = seed_key(&pool, "admin", &["*"]).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/api-keys"))
        .header("X-API-Key", &key)
        .json(&serde_json::json!({"name": "", "scopes": ["events.read"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], false);
}
