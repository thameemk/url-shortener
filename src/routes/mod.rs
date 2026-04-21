mod api;
mod redirect;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get("OK"))
        .route("/{code}", get(redirect::handler))
        .nest("/api", api::router())
        .with_state(state)
}
