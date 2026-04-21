use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::services::url_shortner::list_urls;
use crate::state::AppState;

#[derive(Serialize)]
pub struct UrlEntry {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
    pub short_url: String,
}

pub async fn handler(State(state): State<AppState>) -> impl IntoResponse {
    match list_urls(&state.db).await {
        Ok(urls) => {
            let body: Vec<UrlEntry> = urls
                .into_iter()
                .map(|m| UrlEntry {
                    id: m.id,
                    short_url: format!("http://localhost:8000/{}", m.short_code),
                    short_code: m.short_code,
                    long_url: m.long_url,
                })
                .collect();
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
