use std::sync::Arc;

use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use derive_more::{Display, Error};
use http_body::combinators::UnsyncBoxBody;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::{
    bson::doc,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    email: String,
    password: String,
}

#[derive(Debug, Display, Error)]
pub enum RequestError {
    #[display(fmt = "Invalid credentials")]
    LoginError,
}

impl error::ResponseError for RequestError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            RequestError::LoginError => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AccountDetails {
    name: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReturnedData {
    id: String,
    name: String,
    email: String,
    jwt: String,
}

pub fn generate_token(id: String, secret_key: String, expiry_seconds: i64) -> String {
    let now = Utc::now();
    let exp = now + chrono::Duration::seconds(expiry_seconds);

    let claims = Claims {
        id,
        exp: exp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .unwrap()
}

#[get("/login")]
pub async fn login(
    body: web::Json<LoginInfo>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, RequestError> {
    if let Ok(Some(user)) = data
        .user_collection
        .find_one(doc! {"email": &body.email}, None)
        .await
    {
        let user_password = user.get_str("password").unwrap();
        if verify(&body.password, user_password).unwrap() {
            let user_id = user.get_object_id("_id").unwrap().to_string();

            if !fs::try_exists(format!("./files/{}", user_id))
                .await
                .unwrap()
            {
                fs::create_dir(format!("./files/{}", user_id))
                    .await
                    .unwrap();
            }

            let data = ReturnedData {
                id: user_id.clone(),
                email: user.get_str("email").unwrap().to_owned(),
                name: user.get_str("name").unwrap().to_owned(),
                jwt: generate_token(
                    user_id,
                    data.config.jwt_secret.clone(),
                    data.config.jwt_expiration,
                ),
            };

            Ok(HttpResponse::build(StatusCode::OK).json(data))
        } else {
            Err(RequestError::LoginError)
        }
    } else {
        Err(RequestError::LoginError)
    }
}

/* pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AccountDetails>,
) -> Result<Json<ReturnedData>, Json<RequestError>> {
    let client_options = ClientOptions::parse_with_resolver_config(
        &state.config.mongodb_uri,
        ResolverConfig::cloudflare(),
    )
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
        jwt: generate_token(
            entry_id,
            state.config.jwt_secret.clone(),
            state.config.jwt_expiration,
        ),
    }))
} */
