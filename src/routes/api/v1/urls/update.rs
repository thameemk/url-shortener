use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::services::url_shortner::update_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub long_url: String,
}

#[derive(Serialize)]
pub struct UpdateResponse {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateRequest>,
) -> impl IntoResponse {
    match update_url(&state.db, id, &body.long_url).await {
        Ok(Some(model)) => (
            StatusCode::OK,
            Json(UpdateResponse {
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
