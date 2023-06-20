use axum::{
    extract::{State, TypedHeader},
    headers::authorization::{Authorization, Bearer},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    routes::auth::{Claims, RequestError},
    AppState,
};

pub async fn authentication_middleware<B>(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    if let Ok(token_data) = decode::<Claims>(
        auth.token(),
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        let id = token_data.claims.id;
        request.extensions_mut().insert(id);
        let response = next.run(request).await;

        response
    } else {
        let error_response =
            RequestError::new("Invalid Json Web Token", StatusCode::UNAUTHORIZED).make_response();

        error_response
    }
}
