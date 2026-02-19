use std::{future::Ready, rc::Rc};

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use uuid::Uuid;

pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RequestIdMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
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
        let request_id = Uuid::new_v4().to_string();
        req.extensions_mut().insert(request_id.clone());

        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

pub fn get_request_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}
