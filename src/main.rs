mod middleware;
mod models;
mod routes;
mod services;
mod state;

use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_url_shortner=debug,tower_http=debug".parse().unwrap()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let app = create_router(AppState { db });

    let listener = TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind port");
    println!("Server running on http://127.0.0.1:8000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}
