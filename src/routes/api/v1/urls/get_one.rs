use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::services::url_shortner::get_url_by_id;
use crate::state::AppState;

#[derive(Serialize)]
pub struct UrlResponse {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match get_url_by_id(&state.db, id).await {
        Ok(Some(model)) => (
            StatusCode::OK,
            Json(UrlResponse {
                id: model.id,
                short_code: model.short_code,
                long_url: model.long_url,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "URL not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
