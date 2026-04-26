//! Rate limiter by key (IP or other) with sliding window and 429 response.
use crate::utils::trad::t;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::interval;

/// Entry per key: (request count in window, window start)
type Store = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

/// Configurable rate limiter — sliding window per key (IP or other)
#[derive(Clone)]
pub struct RateLimiter {
    store: Store,
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Window duration
    pub window: Duration,
}

impl RateLimiter {
    /// Creates a rate limiter with default values (60 Req / 60 s).
    ///
    /// # Example
    /// ```rust,ignore
    /// RateLimiter::new()
    ///     .max_requests(100)
    ///     .retry_after(60)
    /// ```
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_requests: 60,
            window: Duration::from_secs(60),
        }
    }

    /// Maximum number of requests allowed in the window
    #[must_use]
    pub fn max_requests(mut self, max: u32) -> Self {
        self.max_requests = max;
        self
    }

    /// Window duration in seconds
    #[must_use]
    pub fn retry_after(mut self, secs: u64) -> Self {
        self.window = Duration::from_secs(secs);
        self
    }

    /// Spawns a Tokio task that periodically purges expired entries.
    /// Should be called once at application startup.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.store.clone();
        let window = self.window;
        tokio::spawn(async move {
            let mut ticker = interval(period);
            loop {
                ticker.tick().await;
                let mut guard = match store.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                let now = Instant::now();
                guard.retain(|_, (_, start)| now.duration_since(*start) < window);
            }
        });
    }

    /// Seconds remaining before window reset for this key.
    /// Returns `0` if the key is unknown or if the window is already expired.
    #[must_use]
    pub fn retry_after_secs(&self, key: &str) -> u64 {
        let store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        match store.get(key) {
            Some((_, start)) => {
                let interval = Instant::now().duration_since(*start);
                self.window.saturating_sub(interval).as_secs()
            }
            None => 0,
        }
    }

    /// Returns `true` if the key is under the limit, `false` if exceeded
    #[must_use]
    pub fn is_allowed(&self, key: &str) -> bool {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let now = Instant::now();
        let entry = store.entry(key.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) >= self.window {
            // New window
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests {
            entry.0 = entry.0.saturating_add(1);
            true
        } else {
            false
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts the IP key from headers (`X-Forwarded-For`, `X-Real-IP`, fallback `"unknown"`).
///
/// **Pre-requisite: trusted reverse proxy.**
/// This function trusts the `X-Forwarded-For` header as it arrives.
/// Without a front proxy (nginx, Caddy, Cloudflare…) that controls this header,
/// a malicious client can forge it to bypass IP rate limiting.
///
/// For brute-force protection on login, prefer [`LoginGuard`] which
/// limits by username — non-bypassable by IP spoofing.
fn extract_ip(req: &Request<Body>) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Rate limiting middleware — to be applied on sensitive routes (login, etc.)
///
/// # Example
/// ```rust,ignore
/// use runique::prelude::*;
/// use std::sync::Arc;
///
/// let limiter = Arc::new(RateLimiter::new().max_requests(5).retry_after(60));
///
/// Router::new()
///     .route("/login", post(login_view))
///     .layer(axum::middleware::from_fn_with_state(
///         limiter,
///         rate_limit_middleware,
///     ))
/// ```
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let ip = extract_ip(&req);
    if limiter.is_allowed(&ip) {
        next.run(req).await
    } else {
        let retry_after = limiter.retry_after_secs(&ip).to_string();
        (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after)],
            t("html.429_text").into_owned(),
        )
            .into_response()
    }
}
