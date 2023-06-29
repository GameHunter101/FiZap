use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Mutex,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error, Error, FromRequest, HttpMessage, HttpResponse,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::AppState;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct AuthenticationFactory;

impl AuthenticationFactory {
    pub fn new() -> Self {
        AuthenticationFactory {}
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthenticationFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = match req.headers().get("Authorization") {
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

        let srv = self.service.clone();
        async move {
            if let Some(token) = token {
                println!("Hi from response");
                req.extensions_mut()
                    .insert::<String>(token);
            } else {
                println!("Uh oh, bad authorization");
                req.extensions_mut()
                    .insert::<String>("Not authorized".to_string());
            }
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}

pub struct AuthenticationExtractor(String);

impl FromRequest for AuthenticationExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<String>().cloned();
        let result = match value {
            Some(v) => Ok(AuthenticationExtractor(v)),
            None => Err(error::ErrorUnauthorized("Invalid token")),
        };
        ready(result)
    }
}

impl std::ops::Deref  for AuthenticationExtractor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
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
