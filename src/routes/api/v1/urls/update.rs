use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{format_short_url, internal_error, not_found, UrlResponse};
use crate::services::url_shortner::update_url;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateRequest {
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
