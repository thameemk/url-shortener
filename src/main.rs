mod routes;
mod services;
mod state;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS urls (
            id SERIAL PRIMARY KEY,
            short_code VARCHAR(10) UNIQUE NOT NULL,
            long_url TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create urls table");

    let app = create_router(AppState { db: pool });

    let listener = TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind port");
    println!("Server running on http://127.0.0.1:8000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
