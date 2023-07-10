use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use client::{ServerApp, ServerAppProps};
use dotenv::dotenv;
use middleware::AuthenticationFactory;
use routes::auth::{login, signup};
use routes::files::{get_file_count, get_files_indices};
use tokio::fs;
use yew::ServerRenderer;

use crate::middleware::AuthenticationExtractor;

mod routes {
    pub mod auth;
    pub mod files;
}
mod middleware;
mod utils;

use utils::AppState;

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
            .service(web::scope("/account").service(login).service(signup))
            .service(
                web::scope("/api")
                    .wrap(AuthenticationFactory::new())
                    .service(get_file_count)
                    .service(get_files_indices)
                    .service(web::scope("/test").service(api)),
            )
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
