use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::State;
use axum::middleware::from_fn_with_state;
use axum::Extension;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    routing::post,
    Router,
};
use clap::Parser;
use dotenv::dotenv;
use mongodb::bson::Document;
use mongodb::{Collection, Client};
use mongodb::options::{ClientOptions, ResolverConfig};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::middleware::authentication_middleware;
use crate::routes::auth::{login, signup};

mod routes {
    pub mod auth;
}
mod middleware;

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
}

impl AppState {
    async fn init() -> AppState {
        let config = Config::init();
        let client_options =
            ClientOptions::parse_with_resolver_config(&config.mongodb_uri, ResolverConfig::cloudflare())
                .await
                .unwrap();
        let client = Client::with_options(client_options).unwrap();
    
        let user_collection = client
            .database("MuZap")
            .collection::<mongodb::bson::Document>("users");
        AppState {
            config,
            user_collection,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    tracing_subscriber::fmt::init();

    let state = AppState::init().await;

    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login))
        .route("/signup", post(signup))
        .nest(
            "/api",
            Router::new()
                .route("/test", get(handler))
                .route_layer(from_fn_with_state(
                    state.clone(),
                    authentication_middleware,
                )),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(Arc::new(state));

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
