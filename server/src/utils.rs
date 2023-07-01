use std::env;

use actix_web::{HttpResponse, error, http::{header::ContentType, StatusCode}};
use clap::Parser;
use derive_more::{Display, Error};
use mongodb::{
    bson::Document,
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};

#[derive(Parser, Debug, Clone)]
#[clap(name = "server", about = "A file hosting server")]
pub struct Opt {
    #[clap(short = 'l', long = "log", default_value = "debug")]
pub log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    pub addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,

    #[clap(long = "static-dir", default_value = "./dist")]
    pub static_dir: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub mongodb_uri: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

impl Config {
    fn init() -> Self {
        let mongodb_uri = env::var("MONGODB_URI").expect("No mongodb uri found");
        let jwt_secret = env::var("JWT_SECRET").expect("No json web token secret found");
        let jwt_expiration = env::var("JWT_EXPIRATION")
            .expect("No json web token expiration found")
            .parse::<i64>()
            .unwrap();
        Config {
            mongodb_uri,
            jwt_secret,
            jwt_expiration,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub user_collection: Collection<Document>,
    pub opt: Opt,
}

impl AppState {
    pub async fn init() -> AppState {
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

#[derive(Debug, Display, Error)]
pub enum CustomError {
    #[display(fmt = "Invalid credentials")]
    LoginError,
    #[display(fmt = "Invalid JWT")]
    JWTError,
    #[display(fmt = "Request is missing a body")]
    MissingBody,
    #[display(fmt = "Path not found")]
    MissingPath,
}

impl error::ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            CustomError::LoginError => StatusCode::UNAUTHORIZED,
            CustomError::JWTError => StatusCode::UNAUTHORIZED,
            CustomError::MissingBody => StatusCode::BAD_REQUEST,
            CustomError::MissingPath => StatusCode::NOT_FOUND,
        }
    }
}