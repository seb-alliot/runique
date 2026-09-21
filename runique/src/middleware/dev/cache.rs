//! Cache-control middleware — injects `Cache-Control: no-cache` in debug mode to force reloading.
use crate::utils::aliases::AEngine;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

/// Forces `Cache-Control: no-cache, no-store, must-revalidate` (plus `Pragma`
/// and `Expires`) on responses when the app is in debug mode and the request
/// targets localhost, so local development never serves a stale cached page.
pub async fn dev_no_cache_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Check if in debug mode AND on localhost
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

/// Returns `true` if the request's `Host` header starts with `localhost`,
/// `127.0.0.1`, or `[::1]`.
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
