use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Returned when creating a new API key -- includes the plaintext key (shown once).
#[derive(Debug, Serialize)]
pub struct ApiKeyWithPlaintext {
    pub id: Uuid,
    pub name: String,
    pub scopes: Vec<String>,
    pub plaintext_key: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Safe view of an API key (no hash exposed).
#[derive(Debug, Serialize)]
pub struct ApiKeySafe {
    pub id: Uuid,
    pub name: String,
    pub scopes: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ApiKey> for ApiKeySafe {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id,
            name: k.name,
            scopes: k.scopes,
            is_active: k.is_active,
            last_used_at: k.last_used_at,
            created_at: k.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateEventBody {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default = "default_payload")]
    pub payload: serde_json::Value,
}

fn default_payload() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyBody {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventListParams {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
}
