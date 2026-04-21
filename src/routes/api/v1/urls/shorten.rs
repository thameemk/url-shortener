use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Deserialize;
use utoipa::ToSchema;

use super::{conflict, format_short_url, internal_error, UrlResponse};
use crate::services::url_shortner::{create_short_url, ShortUrlError};
use crate::state::AppState;

#[derive(Deserialize, ToSchema)]
pub struct ShortenRequest {
    pub long_url: String,
    pub short_code: Option<String>,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

#[utoipa::path(
    post,
    path = "/api/v1/urls",
    tag = "URLs",
    request_body = ShortenRequest,
    responses(
        (status = 201, description = "URL shortened successfully", body = UrlResponse),
        (status = 409, description = "Custom short code is already taken"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn handler(
    State(state): State<AppState>,
    Json(body): Json<ShortenRequest>,
) -> impl IntoResponse {
    match create_short_url(
        &state.db,
        &body.long_url,
        body.short_code.as_deref(),
        body.expires_at,
    )
    .await
    {
        Ok(model) => (
            StatusCode::CREATED,
            Json(UrlResponse {
                id: model.id,
                short_url: format_short_url(&model.short_code),
                short_code: format!("/{}", model.short_code),
                long_url: model.long_url,
                expires_at: model.expires_at,
            }),
        )
            .into_response(),
        Err(ShortUrlError::CodeTaken) => conflict("short code is already taken"),
        Err(ShortUrlError::Db(e)) => internal_error(e),
    }
}
