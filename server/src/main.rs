use std::env;
use std::net::SocketAddr;

use axum::{response::{Html, IntoResponse}, routing::get, Router};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app = Router::new().route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> impl IntoResponse {
    let development = env::var("DEVELOPMENT")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    if development {
        let file = tokio::fs::read_to_string("./assets/index.html").await.unwrap();
        return Html(file);
    } else {
        return Html(include_str!("../assets/index.html").to_string());
    }
}
