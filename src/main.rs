mod models;
mod routes;
mod services;
mod state;

use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tokio::net::TcpListener;

use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

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
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
