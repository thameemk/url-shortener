pub mod get_one;
pub mod list;
pub mod shorten;
pub mod update;

use axum::{routing::get, Router};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Serialize;
use utoipa::ToSchema;

pub use crate::routes::common::{internal_error, not_found, PaginatedResponse, Pagination};
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct UrlResponse {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
    pub short_url: String,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

#[derive(Serialize, ToSchema)]
pub struct UrlListResponse {
    pub results: Vec<UrlResponse>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

pub fn format_short_url(code: &str) -> String {
    format!("http://localhost:8000/{}", code)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/urls", get(list::handler).post(shorten::handler))
        .route("/urls/{id}", get(get_one::handler).patch(update::handler))
}
