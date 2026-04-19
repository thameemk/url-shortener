mod redirect;
mod shorten;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/url/shorten", post(shorten::handler))
        .route("/url/{code}", get(redirect::handler))
}
