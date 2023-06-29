use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use actix_web::dev::{Service, ServiceRequest};
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::Parser;
use client::{ServerApp, ServerAppProps};
use dotenv::dotenv;
use futures_util::FutureExt;
use middleware::{AuthenticationFactory, AuthenticationMiddleware};
use mongodb::{
    bson::Document,
    options::{ClientOptions, ResolverConfig},
    Client, Collection,
};
use routes::auth::login;
use tokio::fs;
use yew::ServerRenderer;

use crate::middleware::AuthenticationExtractor;

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

#[get("/{tail:.*}")]
async fn app(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let index_html_full =
        fs::read_to_string(&format!("{}/index.html", data.opt.static_dir)).await?;

    let url = req.uri().to_string();

    let content =
        ServerRenderer::<ServerApp>::with_props(move || ServerAppProps { url: url.into() })
            .render()
            .await;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-12")
        .body(index_html_full.replace("<body>", &format!("<body>{}", content))))
}

#[get("/")]
async fn api(auth: AuthenticationExtractor) -> impl Responder {
    HttpResponse::Ok().body(format!("you reached the api, {}", *auth))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            // .wrap(AuthenticationFactory::new())
            .service(web::scope("/api").service(api).service(login))
            .service(actix_files::Files::new(
                &state.opt.static_dir.replace(".", ""),
                &state.opt.static_dir,
            ))
            .service(app)
    })
    .bind(addr)?
    .run()
    .await
}
