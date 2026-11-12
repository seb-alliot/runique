//! Rate limiter by key (IP or other) with a fixed window and 429 response.
use crate::utils::trad::t;
use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode, header},
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

/// Configurable rate limiter — fixed window per key (IP or other): the count
/// starts at a key's first request and is dropped once the window has passed.
#[derive(Clone)]
pub struct RateLimiter {
    store: Store,
    /// Maximum number of requests allowed in the window
    pub max_requests: u32,
    /// Window duration
    pub window: Duration,
    /// If set, only these HTTP methods are counted (others pass through freely)
    pub methods: Option<Vec<Method>>,
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
            methods: None,
        }
    }

    /// Restricts rate limiting to the given HTTP methods.
    /// GET requests are never counted if POST is the only method listed.
    #[must_use]
    pub fn only_methods(mut self, methods: Vec<Method>) -> Self {
        self.methods = Some(methods);
        self
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

    /// Cleanup cadence, capped so expired keys aren't retained for a full
    /// long window. Tokio intervals require a non-zero duration.
    pub(crate) fn cleanup_interval(&self) -> tokio::time::Duration {
        self.window
            .clamp(Duration::from_secs(1), Duration::from_secs(60))
    }

    /// Spawns a Tokio task that periodically purges expired entries.
    /// Should be called once at application startup for manual integrations.
    /// The route extensions and admin builder start it automatically.
    pub fn spawn_cleanup(&self, period: tokio::time::Duration) {
        let store = self.store.clone();
        let window = self.window;
        let period = period.max(tokio::time::Duration::from_secs(1));
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
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        if store
            .get(key)
            .is_some_and(|(_, start)| start.elapsed() >= self.window)
        {
            store.remove(key);
            return 0;
        }
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
        if store
            .get(key)
            .is_some_and(|(_, start)| now.duration_since(*start) >= self.window)
        {
            store.remove(key);
        }
        let entry = store.entry(key.to_string()).or_insert((0, now));

        if entry.0 < self.max_requests {
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

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_requests", &self.max_requests)
            .field("window", &self.window)
            .field("methods", &self.methods)
            .finish()
    }
}

/// The limiter's key for a client. An IPv6 subscriber holds a whole /64:
/// keyed on the full address, each of its addresses would get an allowance of
/// its own, and a row in the store. One key per /64 instead.
fn limiter_key(ip: std::net::IpAddr) -> String {
    // An IPv4 client seen through an IPv6 socket (`::ffff:1.2.3.4`) is an
    // IPv4 client: left as IPv6, all of them would share one /64.
    match ip.to_canonical() {
        std::net::IpAddr::V4(v4) => v4.to_string(),
        std::net::IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

/// The client's address: the one `trusted_proxies` resolved, else the peer's.
fn client_ip(req: &Request<Body>) -> Option<std::net::IpAddr> {
    req.extensions()
        .get::<crate::middleware::security::trusted_proxies::ClientIp>()
        .map(|c| c.0)
        .or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip())
        })
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
    if let Some(ref methods) = limiter.methods
        && !methods.contains(req.method())
    {
        return next.run(req).await;
    }
    let ip = client_ip(&req).map_or_else(|| "unknown".to_string(), limiter_key);
    if limiter.is_allowed(&ip) {
        next.run(req).await
    } else {
        let retry_after_secs = limiter.retry_after_secs(&ip);
        if let Some(level) = crate::utils::runique_log::log_level!(middleware.rate_limit) {
            crate::runique_log!(level, %ip, retry_after = retry_after_secs, "rate limited");
        }
        (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after_secs.to_string())],
            t("html.429_text").into_owned(),
        )
            .into_response()
    }
}

#[cfg(test)]
mod expiration_tests {
    use super::*;

    #[test]
    fn the_cleanup_cadence_follows_the_window_within_bounds() {
        let secs = |window| {
            RateLimiter::new()
                .retry_after(window)
                .cleanup_interval()
                .as_secs()
        };
        assert_eq!(secs(0), 1);
        assert_eq!(secs(30), 30);
        assert_eq!(secs(600), 60);
    }

    // Read straight from the store: `retry_after_secs()` would purge the key itself.
    #[tokio::test]
    async fn the_background_task_removes_expired_keys() {
        let limiter = RateLimiter::new().max_requests(5).retry_after(60);
        {
            let mut store = limiter.store.lock().unwrap();
            store.insert(
                "expired".to_string(),
                (1, Instant::now() - Duration::from_secs(61)),
            );
            store.insert("fresh".to_string(), (1, Instant::now()));
        }

        limiter.spawn_cleanup(tokio::time::Duration::from_secs(1));
        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

        let store = limiter.store.lock().unwrap();
        assert!(!store.contains_key("expired"));
        assert!(store.contains_key("fresh"));
    }

    #[test]
    fn retry_after_removes_an_expired_key() {
        let limiter = RateLimiter::new().max_requests(1).retry_after(0);
        assert!(limiter.is_allowed("expired"));
        assert_eq!(limiter.retry_after_secs("expired"), 0);
        assert!(
            !limiter.store.lock().unwrap().contains_key("expired"),
            "expired entry should be removed when observed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ipv6_client_is_keyed_by_its_64_and_an_ipv4_one_by_its_address() {
        let key = |ip: &str| limiter_key(ip.parse().unwrap());
        assert_eq!(key("203.0.113.7"), "203.0.113.7");
        assert_ne!(key("203.0.113.7"), key("203.0.113.8"));
        // Same /64, two addresses: one key.
        assert_eq!(
            key("2001:db8:1:2::1"),
            key("2001:db8:1:2:ffff:ffff:ffff:ffff")
        );
        assert_eq!(key("2001:db8:1:2::1"), "2001:db8:1:2::/64");
        // The next /64 is another client.
        assert_ne!(key("2001:db8:1:2::1"), key("2001:db8:1:3::1"));
        // IPv4 through an IPv6 socket stays one key per address.
        assert_eq!(key("::ffff:203.0.113.7"), "203.0.113.7");
        assert_ne!(key("::ffff:203.0.113.7"), key("::ffff:203.0.113.8"));
    }

    #[test]
    fn test_is_allowed_basic() {
        let limiter = RateLimiter::new().max_requests(3).retry_after(60);
        assert!(limiter.is_allowed("ip1"));
        assert!(limiter.is_allowed("ip1"));
        assert!(limiter.is_allowed("ip1"));
        assert!(!limiter.is_allowed("ip1"));
        // different key unaffected
        assert!(limiter.is_allowed("ip2"));
    }

    #[test]
    fn test_only_methods_skips_unlisted() {
        let limiter = RateLimiter::new()
            .max_requests(2)
            .retry_after(60)
            .only_methods(vec![Method::POST]);

        // GET is not counted — can call indefinitely
        for _ in 0..10 {
            // simulate method check as the middleware does
            let method = Method::GET;
            let methods = limiter.methods.as_ref().unwrap();
            assert!(!methods.contains(&method), "GET should not be in the list");
        }

        // POST IS counted
        assert!(limiter.is_allowed("ip1"));
        assert!(limiter.is_allowed("ip1"));
        assert!(!limiter.is_allowed("ip1"));
    }

    #[test]
    fn test_only_methods_counts_listed() {
        let limiter = RateLimiter::new()
            .max_requests(1)
            .retry_after(60)
            .only_methods(vec![Method::POST, Method::PUT]);

        let methods = limiter.methods.as_ref().unwrap();
        assert!(methods.contains(&Method::POST));
        assert!(methods.contains(&Method::PUT));
        assert!(!methods.contains(&Method::GET));
        assert!(!methods.contains(&Method::DELETE));
    }

    #[test]
    fn test_no_methods_filter_counts_all() {
        let limiter = RateLimiter::new().max_requests(1).retry_after(60);
        assert!(limiter.methods.is_none());
        assert!(limiter.is_allowed("ip1"));
        assert!(!limiter.is_allowed("ip1"));
    }

    #[test]
    fn test_retry_after_secs_unknown_key() {
        let limiter = RateLimiter::new().max_requests(5).retry_after(60);
        assert_eq!(limiter.retry_after_secs("unknown"), 0);
    }

    #[test]
    fn test_retry_after_secs_known_key() {
        let limiter = RateLimiter::new().max_requests(1).retry_after(60);
        let _ = limiter.is_allowed("ip1");
        let _ = limiter.is_allowed("ip1"); // exceeds limit
        let remaining = limiter.retry_after_secs("ip1");
        assert!(remaining > 0 && remaining <= 60);
    }
}
