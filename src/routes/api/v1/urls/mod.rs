mod get_one;
mod list;
mod shorten;
mod update;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct UrlResponse {
    pub id: i32,
    pub short_code: String,
    pub long_url: String,
    pub short_url: String,
}

pub fn format_short_url(code: &str) -> String {
    format!("http://localhost:8000/{}", code)
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
        Json(serde_json::json!({ "error": "URL not found" })),
    )
        .into_response()
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/urls", get(list::handler).post(shorten::handler))
        .route("/urls/{id}", get(get_one::handler).patch(update::handler))
}
