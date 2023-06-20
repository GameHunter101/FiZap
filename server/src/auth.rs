use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: usize,
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