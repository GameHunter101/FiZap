use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{auth::Claims, Config};

async fn authentication_middleware(
    Extension(auth_header): Extension<Option<String>>,
    State(state): State<Arc<Config>>,
) -> Result<(), StatusCode> {
    match auth_header {
        Some(token) => {
            let validation = Validation::default();
            let decoded_token = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(state.jwt_secret.as_ref()),
                &validation,
            );

            match decoded_token {
                Ok(_) => Ok(()),
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
