use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use auth::generate_token;
use axum::extract::State;
use axum::middleware::from_fn_with_state;
use axum::Extension;
use axum::{
    extract::Json,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    routing::post,
    Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use clap::Parser;
use dotenv::dotenv;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::middleware::authentication_middleware;

mod auth;
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    tracing_subscriber::fmt::init();

    let config = Config::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login))
        .route("/signup", post(signup))
        .nest(
            "/api",
            Router::new()
                .route("/test", get(handler))
                .route_layer(from_fn_with_state(
                    config.clone(),
                    authentication_middleware,
                )),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(Arc::new(config));

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

async fn root(State(state): State<Arc<Config>>) -> impl IntoResponse {
    if state.development {
        let file = tokio::fs::read_to_string("./assets/index.html")
            .await
            .unwrap();
        return Html(file);
    } else {
        return Html(include_str!("../assets/index.html").to_string());
    }
}

#[derive(Deserialize)]
struct LoginInfo {
    email: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct AccountDetails {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct ReturnedData {
    id: String,
    name: String,
    email: String,
    jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestError {
    message: String,
    status_code: u16,
}

impl RequestError {
    pub fn new(message: &str, status_code: StatusCode) -> Self {
        Self {
            message: message.to_owned(),
            status_code: status_code.as_u16(),
        }
    }

    pub fn make_response(&self) -> impl IntoResponse {
        let response = Response::builder()
            .status(self.status_code)
            .body(Json(self).into_response().into_body())
            .unwrap();

        response
    }
}

async fn login(
    State(state): State<Arc<Config>>,
    Json(payload): Json<LoginInfo>,
) -> Result<Json<ReturnedData>, Json<RequestError>> {
    let client_options =
        ClientOptions::parse_with_resolver_config(&state.mongodb_uri, ResolverConfig::cloudflare())
            .await
            .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let users = client
        .database("MuZap")
        .collection::<mongodb::bson::Document>("users");

    if let Ok(Some(user)) = users.find_one(doc! {"email": payload.email}, None).await {
        let user_password = user.get_str("password").unwrap();
        if verify(payload.password, user_password).unwrap() {
            let user_id = user.get_object_id("_id").unwrap().to_string();

            if !fs::try_exists(format!("./files/{}", user_id))
                .await
                .unwrap()
            {
                fs::create_dir(format!("./files/{}", user_id))
                    .await
                    .unwrap();
            }

            return Ok(Json(ReturnedData {
                id: user_id.clone(),
                email: user.get_str("email").unwrap().to_owned(),
                name: user.get_str("name").unwrap().to_owned(),
                jwt: generate_token(user_id, state.jwt_secret.clone(), state.jwt_expiration),
            }));
        } else {
            return Err(Json(RequestError::new(
                "Invalid password",
                StatusCode::UNAUTHORIZED,
            )));
        }
    } else {
        return Err(Json(RequestError::new(
            "User not found",
            StatusCode::NOT_FOUND,
        )));
    }
}

async fn signup(
    State(state): State<Arc<Config>>,
    Json(payload): Json<AccountDetails>,
) -> Result<Json<ReturnedData>, Json<RequestError>> {
    let client_options =
        ClientOptions::parse_with_resolver_config(&state.mongodb_uri, ResolverConfig::cloudflare())
            .await
            .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let users = client
        .database("MuZap")
        .collection::<mongodb::bson::Document>("users");

    let user_params = AccountDetails {
        name: payload.name.clone(),
        email: payload.email.clone(),
        password: hash(payload.password, DEFAULT_COST).unwrap(),
    };

    let new_user = mongodb::bson::to_document(&user_params).unwrap();

    let new_entry = users.insert_one(new_user, None).await.unwrap();
    let entry_id = new_entry.inserted_id.as_object_id().unwrap().to_string();
    fs::create_dir(format!("./files/{}", entry_id))
        .await
        .unwrap();

    Ok(Json(ReturnedData {
        id: entry_id.clone(),
        name: payload.name,
        email: payload.email,
        jwt: generate_token(entry_id, state.jwt_secret.clone(), state.jwt_expiration),
    }))
}

async fn handler(Extension(data): Extension<String>) -> impl IntoResponse {
    format!("Hi, {}", data)
}
