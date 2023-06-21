use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use axum::{
    body::{Body, StreamBody},
    extract::{Query, State},
    http::Request,
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use client::{ServerApp, ServerAppProps};
use dotenv::dotenv;
use futures::stream::{self, StreamExt};
use mongodb::{
    bson::Document,
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::middleware::authentication_middleware;
use crate::routes::auth::{login, signup};

mod routes {
    pub mod auth;
}
mod middleware;

#[derive(Parser, Debug, Clone)]
#[clap(name = "server", about = "A file hosting server")]
pub struct Opt {
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[derive(Clone)]
pub struct Config {
    development: bool,
    mongodb_uri: String,
    jwt_secret: String,
    jwt_expiration: i64,
}

impl Config {
    fn init() -> Self {
        let development = env::var("DEVELOPMENT")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        let mongodb_uri = env::var("MONGODB_URI").expect("No mongodb uri found");
        let jwt_secret = env::var("JWT_SECRET").expect("No json web token secret found");
        let jwt_expiration = env::var("JWT_EXPIRATION")
            .expect("No json web token expiration found")
            .parse::<i64>()
            .unwrap();
        Config {
            development,
            mongodb_uri,
            jwt_secret,
            jwt_expiration,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub user_collection: Collection<Document>,
    pub opt: Opt,
}

impl AppState {
    async fn init() -> AppState {
        let config = Config::init();
        let client_options = ClientOptions::parse_with_resolver_config(
            &config.mongodb_uri,
            ResolverConfig::cloudflare(),
        )
        .await
        .unwrap();
        let client = Client::with_options(client_options).unwrap();

        let user_collection = client
            .database("MuZap")
            .collection::<mongodb::bson::Document>("users");
        let opt = Opt::parse();
        AppState {
            config,
            user_collection,
            opt,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let state = AppState::init().await;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            format!("{},hyper=info,mio=info", &state.opt.log_level),
        )
    }

    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from((
        IpAddr::from_str(&state.opt.addr).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        state.opt.port,
    ));

    let app = Router::new()
        // .fallback(render)
        .route("/", get(render))
        .nest(
            "/api",
            Router::new()
                .route("/login", get(login))
                .route("/signup", post(signup))
                .route("/test", get(handler))
                .route_layer(from_fn_with_state(state.clone(), authentication_middleware)),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(Arc::new(state));

    log::info!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn render(
    Query(queries): Query<HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
    url: Request<Body>,
) -> impl IntoResponse {
    let index_html_full =
        tokio::fs::read_to_string(PathBuf::from(&state.opt.static_dir).join("index.html"))
            .await
            .expect("Failed to read index.html");
    // let index_html_split = index_html_full.split("body").collect::<Vec<_>>();
    // let mut index_html_before = index_html_split[0].to_owned();
    // index_html_before.push_str("body>");
    let (index_html_before, index_html_after) = index_html_full.split_once("<body>").unwrap();
    let mut index_html_before = index_html_before.to_owned();
    index_html_before.push_str("<body>");

    let body_end = index_html_after.split_once("<script").unwrap().1;
    let mut index_html_after = "<script".to_owned();
    index_html_after.push_str(body_end);

    // let mut index_html_after = "</body".to_owned();
    // index_html_after.push_str(index_html_split[2]);

    let url = url.uri().to_string();

    let renderer = yew::ServerRenderer::<ServerApp>::with_props(move || ServerAppProps {
        url: url.into(),
        queries,
    });

    StreamBody::new(
        stream::once(async move { index_html_before })
            .chain(renderer.render_stream())
            .chain(stream::once(async move { index_html_after }))
            .map(Result::<_, Infallible>::Ok),
    )
}

async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if state.config.development {
        let file = tokio::fs::read_to_string("./assets/index.html")
            .await
            .unwrap();
        return Html(file);
    } else {
        return Html(include_str!("../assets/index.html").to_string());
    }
}

async fn handler(Extension(data): Extension<String>) -> impl IntoResponse {
    format!("Hi, {}", data)
}
