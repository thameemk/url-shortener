mod get_one;
mod list;
mod shorten;
mod update;

use axum::{routing::get, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/urls", get(list::handler).post(shorten::handler))
        .route("/urls/{id}", get(get_one::handler).patch(update::handler))
}
