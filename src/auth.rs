use axum::Json;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::models::ApiKey;
use crate::response::ApiResponse;

/// Extractor that validates the X-API-Key header against the database.
pub struct ApiKeyAuth(pub ApiKey);

impl FromRequestParts<PgPool> for ApiKeyAuth {
    type Rejection = (StatusCode, Json<ApiResponse<()>>);

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::error("Missing X-API-Key header")),
                )
            })?;

        let mut hasher = Sha256::new();
        hasher.update(header.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let key: ApiKey = sqlx::query_as(
            "SELECT id, name, key_hash, scopes, is_active, last_used_at, created_at
             FROM api_keys WHERE key_hash = $1 AND is_active = true",
        )
        .bind(&hash)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("Database error")),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("Invalid API key")),
            )
        })?;

        // Update last_used_at (fire and forget)
        let pool_clone = pool.clone();
        let key_id = key.id;
        tokio::spawn(async move {
            let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
                .bind(key_id)
                .execute(&pool_clone)
                .await;
        });

        Ok(ApiKeyAuth(key))
    }
}

/// Check that the given API key has the required scope.
/// Wildcard `*` grants access to all scopes.
pub fn require_scope(
    api_key: &ApiKey,
    scope: &str,
) -> Result<(), (StatusCode, Json<ApiResponse<()>>)> {
    if api_key.scopes.contains(&"*".to_string()) || api_key.scopes.contains(&scope.to_string()) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(format!(
                "Missing required scope: {scope}"
            ))),
        ))
    }
}
