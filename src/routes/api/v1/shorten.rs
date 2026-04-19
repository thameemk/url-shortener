use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::services::url_shortner::create_short_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ShortenRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct ShortenResponse {
    pub short_url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(body): Json<ShortenRequest>,
) -> impl IntoResponse {
    match create_short_url(&state.db, &body.url).await {
        Ok(code) => {
            let short_url = format!("http://localhost:8000/{}", code);
            (StatusCode::CREATED, Json(ShortenResponse { short_url })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
