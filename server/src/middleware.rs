use axum::{
    extract::{State, TypedHeader},
    headers::authorization::{Authorization, Bearer},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{auth::Claims, Config, RequestError};

pub async fn authentication_middleware<B>(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<Config>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    if let Ok(token_data) = decode::<Claims>(
        auth.token(),
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        let id = token_data.claims.id;
        request.extensions_mut().insert(id);
        let response = next.run(request).await;

        response
    } else {
        let error_response = Json(RequestError::new(
            "Invalid Json Web Token",
            StatusCode::UNAUTHORIZED,
        ))
        .into_response();

        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(error_response.into_body())
            .unwrap();
    }
}
