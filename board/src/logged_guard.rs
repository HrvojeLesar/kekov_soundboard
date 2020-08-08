  
use crate::prelude::*;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, dev::Payload};
use futures::future::{ok, Ready};
use futures::future::Either;
use actix_web::FromRequest;

use actix_identity::Identity;

pub struct LoggedGuard;

impl<S> Transform<S> for LoggedGuard
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = LoggedGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggedGuardMiddleware { service })
    }
}

pub struct LoggedGuardMiddleware<S> {
    service: S,
}

impl<S> Service for LoggedGuardMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let (r, mut payload) = req.into_parts();
        let is_logged: bool = is_logged(&r, &mut payload);
        let req = ServiceRequest::from_parts(r, payload).ok().unwrap();
        if is_logged {
            Either::Left(self.service.call(req))
        } else {
            Either::Right(ok(
                req.into_response(
                    HttpResponse::SeeOther()
                        .header(http::header::LOCATION, "/login")
                        .finish()
                        .into_body(),
            )))
        }
    }
}

fn is_logged(req: &HttpRequest, payload: &mut Payload) -> bool {
    if let Ok(id) = Identity::from_request(req, payload).into_inner() {
        if let Some(_) = id.identity() {
            return true;
        }
    }
    return false;
}