use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::header::AUTHORIZATION,
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures::future::LocalBoxFuture;

use crate::{auth::Claims, errors::AppError};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract JWT service from app data
            let jwt_service = req
                .app_data::<actix_web::web::Data<crate::auth::JwtService>>()
                .ok_or_else(|| ErrorUnauthorized("JWT service not configured"))?;

            // Extract token from Authorization header
            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| ErrorUnauthorized("Missing authorization header"))?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| ErrorUnauthorized("Invalid authorization header format"))?;

            // Validate token and extract claims
            let claims = jwt_service
                .validate_token(token)
                .map_err(|_| ErrorUnauthorized("Invalid or expired token"))?;

            // Insert claims into request extensions
            req.extensions_mut().insert(claims);

            // Call the next service
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// Extractor for authenticated user in handlers
pub struct AuthenticatedUser(pub Claims);

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let claims = req
            .extensions()
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()));

        ready(claims.map(AuthenticatedUser))
    }
}
