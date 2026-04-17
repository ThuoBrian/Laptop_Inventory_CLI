use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{HeaderValue, SET_COOKIE},
        Method,
    },
    Error, HttpResponse,
};
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

type LocalBoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a>>;

pub struct Csrf;

impl<S, B> Transform<S, ServiceRequest> for Csrf
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = CsrfMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfMiddleware { service: Rc::new(service) }))
    }
}

pub struct CsrfMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Validate on all POST requests to /ui/...
        if req.method() == Method::POST && req.path().starts_with("/ui/") {
            let cookie_token = req.cookie("csrf_token").map(|c| c.value().to_string());
            let header_token = req
                .headers()
                .get("X-CSRF-Token")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let valid = match (&cookie_token, &header_token) {
                (Some(c), Some(h)) => c == h,
                _ => false,
            };

            if !valid {
                let (http_req, _payload) = req.into_parts();
                let resp = HttpResponse::Forbidden()
                    .content_type("text/plain")
                    .body("CSRF token missing or invalid");
                return Box::pin(async move {
                    Ok(ServiceResponse::new(http_req, resp).map_into_right_body())
                });
            }
        }

        // Set the cookie on first visit (double-submit cookie pattern).
        let needs_cookie = req.cookie("csrf_token").is_none();
        let svc = self.service.clone();
        Box::pin(async move {
            let mut res = svc.call(req).await?.map_into_left_body();
            if needs_cookie {
                let token = Uuid::new_v4().to_string();
                if let Ok(val) = HeaderValue::from_str(&format!(
                    "csrf_token={}; SameSite=Strict; Path=/",
                    token
                )) {
                    res.headers_mut().insert(SET_COOKIE, val);
                }
            }
            Ok(res)
        })
    }
}
