use actix_session::SessionExt;
use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::{self, StatusCode},
    web::Json,
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use serde_json::json;
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

fn is_public_path(path: &str) -> bool {
    if !path.starts_with("/api") {
        return true;
    }

    if path.starts_with("/api/auth") && !path.starts_with("/api/auth/user") {
        return true;
    }

    false
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();

        let user_id = req.get_session().get::<Uuid>("user_id").unwrap_or(None);

        if user_id.is_none() && !is_public_path(path) {
            Box::pin(async move {
                let request = req.into_parts().0;
                let response = HttpResponse::Unauthorized()
                    .json(json!({
                        "message":"Signin required."
                    }))
                    .map_into_right_body();

                let res = ServiceResponse::new(request, response);

                Ok(res)
            })
        } else {
            let fut = self.service.call(req);
            Box::pin(async move {
                let tmp: ServiceResponse<B> = fut.await?;
                Ok(tmp.map_into_left_body())
            })
        }
    }
}
