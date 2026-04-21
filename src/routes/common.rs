use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    1
}
fn default_page_size() -> u64 {
    20
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub results: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(results: Vec<T>, page: u64, page_size: u64, total: u64) -> Self {
        let total_pages = total.div_ceil(page_size);
        Self {
            results,
            page,
            page_size,
            total,
            total_pages,
        }
    }
}

pub fn internal_error(e: impl std::fmt::Display) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    )
        .into_response()
}

pub fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not found" })),
    )
        .into_response()
}
