use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::{internal_error, not_found};
use crate::services::url_shortner::get_url_analytics;
use crate::state::AppState;

#[derive(Deserialize, IntoParams)]
pub struct AnalyticsParams {
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    50
}

#[derive(Serialize, ToSchema)]
pub struct ClickResponse {
    pub id: i32,
    pub clicked_at: DateTimeWithTimeZone,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AnalyticsResponse {
    pub url_id: i32,
    pub total_clicks: u64,
    pub clicks: Vec<ClickResponse>,
}

#[utoipa::path(
    get,
    path = "/api/v1/urls/{id}/analytics",
    tag = "URLs",
    params(
        ("id" = i32, Path, description = "URL record ID"),
        AnalyticsParams,
    ),
    responses(
        (status = 200, description = "Analytics for the URL", body = AnalyticsResponse),
        (status = 404, description = "URL not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(params): Query<AnalyticsParams>,
) -> impl IntoResponse {
    let limit = params.limit.clamp(1, 1000);
    match get_url_analytics(&state.db, id, limit).await {
        Ok(Some(analytics)) => {
            let clicks = analytics
                .clicks
                .into_iter()
                .map(|c| ClickResponse {
                    id: c.id,
                    clicked_at: c.clicked_at,
                    ip_address: c.ip_address,
                    user_agent: c.user_agent,
                    referer: c.referer,
                })
                .collect();
            (
                StatusCode::OK,
                Json(AnalyticsResponse {
                    url_id: id,
                    total_clicks: analytics.total_clicks,
                    clicks,
                }),
            )
                .into_response()
        }
        Ok(None) => not_found(),
        Err(e) => internal_error(e),
    }
}
