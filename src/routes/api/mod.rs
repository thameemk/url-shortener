pub mod v1;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest("/v1", v1::router())
}
