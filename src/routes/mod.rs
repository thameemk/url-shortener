mod api;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get("OK"))
        .nest("/api", api::router())
        .with_state(state)
}
