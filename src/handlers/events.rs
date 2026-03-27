use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{ApiKeyAuth, require_scope};
use crate::models::{CreateEventBody, Event, EventListParams};
use crate::response::ApiResponse;

pub async fn create_event(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Json(body): Json<CreateEventBody>,
) -> Result<(StatusCode, Json<ApiResponse<Event>>), (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "events.write")?;

    if body.event_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error_with_details(
                "Validation failed",
                vec!["type is required".into()],
            )),
        ));
    }

    let event: Event = sqlx::query_as(
        "INSERT INTO events (type, payload) VALUES ($1, $2)
         RETURNING id, type, payload, status, created_at, updated_at, deleted_at",
    )
    .bind(&body.event_type)
    .bind(&body.payload)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create event: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to create event")),
        )
    })?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(event))))
}

pub async fn list_events(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Query(params): Query<EventListParams>,
) -> Result<Json<ApiResponse<Vec<Event>>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "events.read")?;

    let events: Vec<Event> = if let Some(ref event_type) = params.event_type {
        sqlx::query_as(
            "SELECT id, type, payload, status, created_at, updated_at, deleted_at
             FROM events WHERE deleted_at IS NULL AND type = $1
             ORDER BY created_at DESC",
        )
        .bind(event_type)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as(
            "SELECT id, type, payload, status, created_at, updated_at, deleted_at
             FROM events WHERE deleted_at IS NULL
             ORDER BY created_at DESC",
        )
        .fetch_all(&pool)
        .await
    }
    .map_err(|e| {
        tracing::error!("Failed to list events: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to list events")),
        )
    })?;

    Ok(Json(ApiResponse::success(events)))
}

pub async fn get_event(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Event>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "events.read")?;

    let event: Event = sqlx::query_as(
        "SELECT id, type, payload, status, created_at, updated_at, deleted_at
         FROM events WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get event: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to get event")),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Event not found")),
        )
    })?;

    Ok(Json(ApiResponse::success(event)))
}

pub async fn delete_event(
    State(pool): State<PgPool>,
    ApiKeyAuth(key): ApiKeyAuth,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    require_scope(&key, "events.write")?;

    let result = sqlx::query(
        "UPDATE events SET deleted_at = NOW(), updated_at = NOW()
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete event: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("Failed to delete event")),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Event not found")),
        ));
    }

    Ok(Json(ApiResponse::success_with_message(
        "deleted",
        "Event soft-deleted",
    )))
}
