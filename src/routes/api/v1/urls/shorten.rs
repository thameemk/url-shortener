use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use super::{format_short_url, internal_error, UrlResponse};
use crate::services::url_shortner::create_short_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ShortenRequest {
    pub long_url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(body): Json<ShortenRequest>,
) -> impl IntoResponse {
    match create_short_url(&state.db, &body.long_url).await {
        Ok(code) => (
            StatusCode::CREATED,
            Json(UrlResponse {
                id: 1,
                short_url: format_short_url(&code),
                short_code: format!("/{}", code),
                long_url: body.long_url,
            }),
        )
            .into_response(),
        Err(e) => internal_error(e),
    }
}
