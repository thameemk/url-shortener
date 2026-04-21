use axum::Router;

use crate::state::AppState;

mod urls;

pub fn router() -> Router<AppState> {
    Router::new().merge(urls::router())
}
