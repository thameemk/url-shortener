use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Deserialize;
use utoipa::ToSchema;

use super::{format_short_url, internal_error, not_found, UrlResponse};
use crate::services::url_shortner::update_url;
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct UpdateRequest {
    pub long_url: String,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/urls/{id}",
    tag = "URLs",
    params(
        ("id" = i32, Path, description = "URL record ID")
    ),
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "URL updated successfully", body = UrlResponse),
        (status = 404, description = "URL not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateRequest>,
) -> impl IntoResponse {
    match update_url(&state.db, id, &body.long_url, body.expires_at).await {
        Ok(Some(model)) => (
            StatusCode::OK,
            Json(UrlResponse {
                id: model.id,
                short_url: format_short_url(&model.short_code),
                short_code: model.short_code,
                long_url: model.long_url,
                expires_at: model.expires_at,
            }),
        )
            .into_response(),
        Ok(None) => not_found(),
        Err(e) => internal_error(e),
    }
}
