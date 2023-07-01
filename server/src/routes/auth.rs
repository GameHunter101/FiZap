use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::{
    bson::doc,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{utils::CustomError, AppState};

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
) -> Result<HttpResponse, CustomError> {
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
            Err(CustomError::LoginError)
        }
    } else {
        Err(CustomError::LoginError)
    }
}

#[post("/signup")]
pub async fn signup(
    data: web::Data<AppState>,
    body: web::Json<AccountDetails>,
) -> Result<HttpResponse, CustomError> {
    let client_options = ClientOptions::parse_with_resolver_config(
        &data.config.mongodb_uri,
        ResolverConfig::cloudflare(),
    )
    .await
    .unwrap();
    let client = Client::with_options(client_options).unwrap();

    let users = client
        .database("MuZap")
        .collection::<mongodb::bson::Document>("users");

    let user_params = AccountDetails {
        name: body.name.clone(),
        email: body.email.clone(),
        password: hash(body.password.clone(), DEFAULT_COST).unwrap(),
    };

    let new_user = mongodb::bson::to_document(&user_params).unwrap();

    let new_entry = users.insert_one(new_user, None).await.unwrap();
    let entry_id = new_entry.inserted_id.as_object_id().unwrap().to_string();
    fs::create_dir(format!("./files/{}", entry_id))
        .await
        .unwrap();

    let data = ReturnedData {
        id: entry_id.clone(),
        name: body.name.clone(),
        email: body.email.clone(),
        jwt: generate_token(
            entry_id,
            data.config.jwt_secret.clone(),
            data.config.jwt_expiration,
        ),
    };

    Ok(HttpResponse::build(StatusCode::OK).json(data))
}
