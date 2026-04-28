use crate::config::config_env::Config;
use crate::core::entities::auth::UserClaims;
use crate::validators::common::is_public_route;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    http::Method,
    web,
};
use futures::future::{LocalBoxFuture, Ready, err, ok};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use subtle::ConstantTimeEq;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService { service })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        if req.method() == Method::OPTIONS {
            return Box::pin(self.service.call(req));
        }

        let Some(config) = req.app_data::<web::Data<Config>>() else {
            return Box::pin(err(ErrorInternalServerError("missing application config")));
        };

        if let Err(e) = self.verify_api_key(&req, config) {
            return Box::pin(err(e));
        }

        if is_public_route(req.path()) {
            return Box::pin(self.service.call(req));
        }

        match self.verify_jwt_token(&req, config) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
                Box::pin(self.service.call(req))
            }
            Err(e) => Box::pin(err(e)),
        }
    }
}

impl<S> AuthMiddlewareService<S> {
    fn verify_api_key(&self, req: &ServiceRequest, config: &Config) -> Result<(), Error> {
        if req.path().starts_with("/api/swagger") || req.path().starts_with("/logstreamer") {
            return Ok(());
        }

        let Some(header) = req.headers().get("api_key") else {
            return Err(ErrorUnauthorized("empty api_key"));
        };
        let provided = header.as_bytes();
        let expected = config.api_key.as_bytes();
        if provided.len() == expected.len() && provided.ct_eq(expected).into() {
            Ok(())
        } else {
            Err(ErrorUnauthorized("wrong api_key"))
        }
    }

    fn verify_jwt_token(&self, req: &ServiceRequest, config: &Config) -> Result<UserClaims, Error> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();

        if !auth_header.starts_with("Bearer ") {
            return Err(ErrorUnauthorized("Invalid authorization header"));
        }

        let token = &auth_header[7..];
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&config.jwt_issuer]);
        validation.set_audience(&[&config.jwt_audience]);

        decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| ErrorUnauthorized("Invalid token"))
    }
}
