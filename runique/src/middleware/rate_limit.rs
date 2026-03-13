use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Entrée par clé : (nombre de requêtes dans la fenêtre, début de la fenêtre)
type Store = Arc<Mutex<HashMap<String, (u32, Instant)>>>;

/// Rate limiter configurable — fenêtre glissante par clé (IP ou autre)
#[derive(Clone)]
pub struct RateLimiter {
    store: Store,
    /// Nombre maximal de requêtes autorisées dans la fenêtre
    pub max_requests: u32,
    /// Durée de la fenêtre
    pub window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Construit depuis les variables d'environnement :
    /// `RUNIQUE_RATE_LIMIT_REQUESTS` (défaut : 60)
    /// `RUNIQUE_RATE_LIMIT_WINDOW_SECS` (défaut : 60)
    pub fn from_env() -> Self {
        let max = std::env::var("RUNIQUE_RATE_LIMIT_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let window = std::env::var("RUNIQUE_RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        Self::new(max, window)
    }

    /// Retourne `true` si la clé est sous la limite, `false` si dépassée
    pub fn is_allowed(&self, key: &str) -> bool {
        let mut store = match self.store.lock() {
            Ok(s) => s,
            Err(p) => p.into_inner(),
        };
        let now = Instant::now();
        let entry = store.entry(key.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) >= self.window {
            // Nouvelle fenêtre
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

/// Extrait la clé IP depuis les headers (`X-Forwarded-For`, `X-Real-IP`, fallback `"unknown"`)
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

/// Middleware de rate limiting — à appliquer sur les routes sensibles (login, etc.)
///
/// # Exemple
/// ```rust,ignore
/// use runique::prelude::*;
/// use std::sync::Arc;
///
/// let limiter = Arc::new(RateLimiter::new(5, 60)); // 5 req/min par IP
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
        (StatusCode::TOO_MANY_REQUESTS, "Too many requests").into_response()
    }
}
