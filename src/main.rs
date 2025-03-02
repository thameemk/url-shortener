use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Define routes
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind port");
    println!("🚀 Server running on http://127.0.0.1:8000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
