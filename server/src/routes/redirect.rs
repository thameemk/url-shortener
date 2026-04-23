use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::services::url_shortner::{record_click, resolve_short_url, ClickRecord, ResolveResult};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/{code}",
    tag = "Redirect",
    params(
        ("code" = String, Path, description = "Short URL code")
    ),
    responses(
        (status = 302, description = "Redirects to the original URL"),
        (status = 404, description = "Short URL not found"),
        (status = 410, description = "Short URL has expired"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn handler(
    State(state): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match resolve_short_url(&state.db, &code).await {
        Ok(ResolveResult::Found(long_url, url_id)) => {
            let ip_address = headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or(s).trim().to_owned());
            let user_agent = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(ToOwned::to_owned);
            let referer = headers
                .get("referer")
                .and_then(|v| v.to_str().ok())
                .map(ToOwned::to_owned);

            let _ = record_click(
                &state.db,
                url_id,
                ClickRecord { ip_address, user_agent, referer },
            )
            .await;

            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                axum::http::header::LOCATION,
                HeaderValue::from_str(&long_url).unwrap(),
            );
            (StatusCode::FOUND, response_headers).into_response()
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
