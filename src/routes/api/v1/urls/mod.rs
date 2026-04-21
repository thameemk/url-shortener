mod get_one;
mod list;
mod shorten;
mod update;

use axum::{routing::get, Router};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::Serialize;

pub use crate::routes::common::{internal_error, not_found, PaginatedResponse, Pagination};
use crate::state::AppState;

#[derive(Serialize)]
pub struct UrlResponse {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
    pub short_url: String,
    pub expires_at: Option<DateTimeWithTimeZone>,
}

pub fn format_short_url(code: &str) -> String {
    format!("http://localhost:8000/{}", code)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/urls", get(list::handler).post(shorten::handler))
        .route("/urls/{id}", get(get_one::handler).patch(update::handler))
}
