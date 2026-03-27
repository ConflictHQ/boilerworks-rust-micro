use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{ApiKeyAuth, require_scope};
use crate::models::{ApiKey, ApiKeySafe, ApiKeyWithPlaintext, CreateApiKeyBody};
use crate::response::ApiResponse;

fn generate_plaintext_key() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::rng().random::<u8>()).collect();
    format!("bw_{}", hex::encode(bytes))
}

fn hash_key(plaintext: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn create_api_key(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Json(body): Json<CreateApiKeyBody>,
) -> Result<(StatusCode, Json<ApiResponse<ApiKeyWithPlaintext>>), (StatusCode, Json<ApiResponse<()>>)>
{
    require_scope(&key, "keys.manage")?;

    if body.name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error_with_details(
                "Validation failed",
                vec!["name is required".into()],
            )),
        ));
    }

    let plaintext = generate_plaintext_key();
    let key_hash = hash_key(&plaintext);
    let scopes = if body.scopes.is_empty() {
        vec!["events.read".to_string()]
    } else {
        body.scopes
    };

    let row: ApiKey = sqlx::query_as(
        "INSERT INTO api_keys (name, key_hash, scopes)
         VALUES ($1, $2, $3)
         RETURNING id, name, key_hash, scopes, is_active, last_used_at, created_at",
    )
    .bind(&body.name)
    .bind(&key_hash)
    .bind(&scopes)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create API key: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to create API key")),
        )
    })?;

    let result = ApiKeyWithPlaintext {
        id: row.id,
        name: row.name,
        scopes: row.scopes,
        plaintext_key: plaintext,
        created_at: row.created_at,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success_with_message(
            result,
            "Store the plaintext_key now -- it will not be shown again",
        )),
    ))
}

pub async fn list_api_keys(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
) -> Result<Json<ApiResponse<Vec<ApiKeySafe>>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "keys.manage")?;

    let keys: Vec<ApiKey> = sqlx::query_as(
        "SELECT id, name, key_hash, scopes, is_active, last_used_at, created_at
         FROM api_keys WHERE is_active = true
         ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list API keys: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to list API keys")),
        )
    })?;

    let safe_keys: Vec<ApiKeySafe> = keys.into_iter().map(ApiKeySafe::from).collect();
    Ok(Json(ApiResponse::success(safe_keys)))
}

pub async fn revoke_api_key(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "keys.manage")?;

    let result =
        sqlx::query("UPDATE api_keys SET is_active = false WHERE id = $1 AND is_active = true")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to revoke API key: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("Failed to revoke API key")),
                )
            })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("API key not found")),
        ));
    }

    Ok(Json(ApiResponse::success_with_message(
        "revoked",
        "API key revoked",
    )))
}
