use std::{
    future::{ready, Ready},
    rc::Rc,
    sync::Mutex,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, FromRequest, HttpMessage,
};
use futures_util::{future::LocalBoxFuture, FutureExt};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{routes::auth::Claims, utils::CustomError, AppState};

#[derive(Debug, Clone)]
pub struct AuthenticationData {
    jwt: String,
}

impl AuthenticationData {
    fn new() -> Self {
        AuthenticationData { jwt: String::new() }
    }

    fn update_token(&mut self, jwt: String) {
        self.jwt = jwt;
    }

    fn extract_id(&self, secret: &[u8]) -> Option<String> {
        if let Ok(token_data) = decode::<Claims>(
            &self.jwt,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        ) {
            return Some(token_data.claims.id);
        }
        None
    }
}

pub struct AuthenticationFactory {}

impl AuthenticationFactory {
    pub fn new() -> Self {
        AuthenticationFactory {}
    }
}

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
            auth_data: Rc::new(Mutex::new(AuthenticationData::new())),
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    auth_data: Rc<Mutex<AuthenticationData>>,
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
        let srv = self.service.clone();
        let mut auth_data = self.auth_data.lock().unwrap();

        match req.headers().get("Authorization") {
            Some(auth) => {
                let auth_str = auth.to_str().unwrap();
                if auth_str.contains("Bearer ") {
                    let token = auth_str.to_string().replace("Bearer ", "");
                    if token.len() > 0 {
                        (*auth_data).update_token(token);
                    }
                }
            }
            None => {}
        };

        let extension_data = (*auth_data).clone();
        async move {
            req.extensions_mut()
                .insert::<AuthenticationData>(extension_data);
            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}

pub struct AuthenticationExtractor(String);

impl FromRequest for AuthenticationExtractor {
    type Error = CustomError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let value = req.extensions().get::<AuthenticationData>().cloned();
        let result = match value {
            Some(v) => {
                let extract_result = v.extract_id(
                    req.app_data::<web::Data<AppState>>()
                        .unwrap()
                        .config
                        .jwt_secret
                        .as_ref(),
                );
                match extract_result {
                    Some(id) => Ok(AuthenticationExtractor(id)),
                    None => Err(CustomError::JWTError),
                }
            }
            None => Err(CustomError::JWTError),
        };
        ready(result)
    }
}

impl std::ops::Deref for AuthenticationExtractor {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
