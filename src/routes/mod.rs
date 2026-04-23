mod api;
pub mod common;
pub mod redirect;

use axum::{middleware, response::Html, routing::get, Json, Router};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

use crate::{
    middleware::rate_limit::{new_rate_limiter, rate_limit_middleware},
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        redirect::handler,
        api::v1::urls::shorten::handler,
        api::v1::urls::list::handler,
        api::v1::urls::get_one::handler,
        api::v1::urls::update::handler,
        api::v1::urls::analytics::handler,
    ),
    components(schemas(
        api::v1::urls::UrlResponse,
        api::v1::urls::UrlListResponse,
        api::v1::urls::shorten::ShortenRequest,
        api::v1::urls::update::UpdateRequest,
        api::v1::urls::analytics::AnalyticsResponse,
        api::v1::urls::analytics::ClickResponse,
    )),
    tags(
        (name = "Redirect", description = "Redirect short URLs to their original destination"),
        (name = "URLs", description = "URL shortening and management")
    ),
    info(
        title = "URL Shortener API",
        version = "1.0.0",
        description = "A simple URL shortener service"
    )
)]
struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn scalar_ui() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html>
  <head>
    <title>URL Shortener API</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <script id="api-reference" data-url="/api-doc/openapi.json"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>"#,
    )
}

pub fn create_router(state: AppState) -> Router {
    let api_limiter = new_rate_limiter(state.config.api_rate_limit);
    let global_limiter = new_rate_limiter(state.config.global_rate_limit);

    let api_routes = api::router().layer(middleware::from_fn(move |req, next| {
        let limiter = api_limiter.clone();
        async move { rate_limit_middleware(limiter, req, next).await }
    }));

    Router::new()
        .route("/docs", get(scalar_ui))
        .route("/api-doc/openapi.json", get(openapi_json))
        .route("/", get(|| async { "OK" }))
        .route("/{code}", get(redirect::handler))
        .nest("/api", api_routes)
        .layer(middleware::from_fn(move |req, next| {
            let limiter = global_limiter.clone();
            async move { rate_limit_middleware(limiter, req, next).await }
        }))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
