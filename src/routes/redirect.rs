use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::services::url_shortner::{resolve_short_url, ResolveResult};
use crate::state::AppState;

pub async fn handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    match resolve_short_url(&state.db, &code).await {
        Ok(ResolveResult::Found(long_url)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::LOCATION,
                HeaderValue::from_str(&long_url).unwrap(),
            );
            (StatusCode::FOUND, headers).into_response()
        }
        Ok(ResolveResult::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "short URL not found" })),
        )
            .into_response(),
        Ok(ResolveResult::Expired) => (
            StatusCode::GONE,
            Json(serde_json::json!({ "error": "short URL has expired" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
