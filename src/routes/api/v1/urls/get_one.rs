use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::{format_short_url, internal_error, not_found, UrlResponse};
use crate::services::url_shortner::get_url_by_id;
use crate::state::AppState;

pub async fn handler(State(state): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    match get_url_by_id(&state.db, id).await {
        Ok(Some(model)) => (
            StatusCode::OK,
            Json(UrlResponse {
                id: model.id,
                short_url: format_short_url(&model.short_code),
                short_code: model.short_code,
                long_url: model.long_url,
            }),
        )
            .into_response(),
        Ok(None) => not_found(),
        Err(e) => internal_error(e),
    }
}
