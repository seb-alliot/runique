use crate::utils::aliases::AEngine;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

pub async fn dev_no_cache_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Vérifier si on est en mode debug ET sur localhost
    let should_no_cache = engine.config.debug && is_localhost(&req);

    let mut response = next.run(req).await;

    if should_no_cache {
        let headers = response.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        );
        headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
        headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
    }

    response
}

pub fn is_localhost(req: &Request<Body>) -> bool {
    req.headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|host| {
            host.starts_with("localhost")
                || host.starts_with("127.0.0.1")
                || host.starts_with("[::1]")
        })
}
