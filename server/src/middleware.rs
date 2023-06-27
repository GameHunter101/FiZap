use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::AppState;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct SayHiFactory;

impl SayHiFactory {
    pub fn new() -> Self {
        SayHiFactory {}
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for SayHiFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SayHiMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SayHiMiddleware { service }))
    }
}

pub struct SayHiMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SayHiMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let bearer_token: Option<String> = match req.headers().get("Authorization") {
            Some(auth) => {
                let auth_str = auth.to_str().unwrap();
                if auth_str.contains("Bearer ") {
                    let token = auth_str.to_string().replace("Bearer ", "");
                    if token.len() > 0 {
                        Some(token)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        };

        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(token) = bearer_token {
                let res = fut.await?;
    
                println!("Hi from response");
                Ok(res)
            } else {
                let err = Error::from("test".into());
                println!("Uh oh, bad authorization");
                Err(err)
            }
        })
    }
}

/* pub async fn authentication_middleware<B>(
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
 */
