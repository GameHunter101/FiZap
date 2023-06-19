use std::env;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use axum::body::{boxed, Body};
use axum::http::{Response, StatusCode};
use axum::response::Html;
use axum::{response::IntoResponse, routing::get, Router};
use chrono::{TimeZone, Utc};
use clap::Parser;
use dotenv::dotenv;
use mongodb::bson::doc;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client,
};
use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Parser, Debug)]
#[clap(name = "server", about = "A file hosting server")]
struct Opt {
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    #[clap(long = "static-dir", default_value = "../client/dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let addr = SocketAddr::from((
        IpAddr::from_str(&opt.addr).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    let development = env::var("DEVELOPMENT")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    if development {
        let file = tokio::fs::read_to_string("./assets/index.html")
            .await
            .unwrap();
        return Html(file);
    } else {
        return Html(include_str!("../assets/index.html").to_string());
    }
}

async fn login() -> impl IntoResponse {
    let client_uri = env::var("MONGODB_URI").expect("No MongoDb URI found");

    let client_options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await
            .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let users = client
        .database("MuZap")
        .collection::<mongodb::bson::Document>("users");

    let user = users
        .find_one(doc! {"email": "liorscarmeli@gmail.com"}, None)
        .await
        .unwrap()
        .expect("User not found");
    format!("{user}")
}