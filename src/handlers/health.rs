use axum::Json;

use crate::response::ApiResponse;

pub async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}
