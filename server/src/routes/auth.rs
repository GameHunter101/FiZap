use std::sync::Arc;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
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

    pub fn make_response(&self) -> Response<UnsyncBoxBody<axum::body::Bytes, axum::Error>> {
        let response = Response::builder()
            .status(self.status_code)
            .body(Json(self.message.clone()).into_response().into_body())
            .unwrap();

        response
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

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginInfo>,
) -> Result<Json<ReturnedData>, Json<RequestError>> {
    if let Ok(Some(user)) = state
        .user_collection
        .find_one(doc! {"email": payload.email}, None)
        .await
    {
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
                jwt: generate_token(
                    user_id,
                    state.config.jwt_secret.clone(),
                    state.config.jwt_expiration,
                ),
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

pub async fn signup(
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
}
