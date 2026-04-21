use super::{format_short_url, internal_error, PaginatedResponse, Pagination, UrlResponse};
use crate::services::url_shortner::list_urls;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn handler(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> impl IntoResponse {
    let page_size = pagination.page_size.clamp(1, 100);
    let page = pagination.page.max(1);

    match list_urls(&state.db, page, page_size).await {
        Ok((urls, total)) => {
            let results: Vec<UrlResponse> = urls
                .into_iter()
                .map(|m| UrlResponse {
                    short_url: format_short_url(&m.short_code),
                    id: m.id,
                    short_code: m.short_code,
                    long_url: m.long_url,
                    expires_at: m.expires_at,
                })
                .collect();
            let body = PaginatedResponse::new(results, page, page_size, total);
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(e) => internal_error(e),
    }
}
