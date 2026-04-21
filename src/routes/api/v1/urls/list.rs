use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::{format_short_url, internal_error, UrlResponse};
use crate::services::url_shortner::list_urls;
use crate::state::AppState;

pub async fn handler(State(state): State<AppState>) -> impl IntoResponse {
    match list_urls(&state.db).await {
        Ok(urls) => {
            let body: Vec<UrlResponse> = urls
                .into_iter()
                .map(|m| UrlResponse {
                    short_url: format_short_url(&m.short_code),
                    id: m.id,
                    short_code: m.short_code,
                    long_url: m.long_url,
                })
                .collect();
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => internal_error(e),
    }
}
