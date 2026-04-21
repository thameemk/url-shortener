use axum::Router;

use crate::state::AppState;

pub mod urls;

pub fn router() -> Router<AppState> {
    Router::new().merge(urls::router())
}
