//! Rate limiter par clé (IP ou autre) avec fenêtre glissante et réponse 429.
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
    /// Crée un rate limiter avec les valeurs par défaut (60 req / 60 s).
    ///
    /// # Exemple
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

    /// Nombre maximal de requêtes autorisées dans la fenêtre
    #[must_use]
    pub fn max_requests(mut self, max: u32) -> Self {
        self.max_requests = max;
        self
    }

    /// Durée de la fenêtre en secondes
    #[must_use]
    pub fn retry_after(mut self, secs: u64) -> Self {
        self.window = Duration::from_secs(secs);
        self
    }

    /// Spawne une tâche Tokio qui purge périodiquement les entrées expirées.
    /// À appeler une fois au démarrage de l'application.
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

    /// Secondes restantes avant la réinitialisation de la fenêtre pour cette clé.
    /// Retourne `0` si la clé est inconnue ou si la fenêtre est déjà expirée.
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

    /// Retourne `true` si la clé est sous la limite, `false` si dépassée
    #[must_use]
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

/// Extrait la clé IP depuis les headers (`X-Forwarded-For`, `X-Real-IP`, fallback `"unknown"`).
///
/// **Pré-requis : reverse proxy de confiance.**
/// Cette fonction fait confiance au header `X-Forwarded-For` tel qu'il arrive.
/// Sans proxy en amont (nginx, Caddy, Cloudflare…) qui contrôle ce header,
/// un client malveillant peut le forcer pour contourner le rate limiting par IP.
///
/// Pour la protection brute-force sur le login, préférez [`LoginGuard`] qui
/// limite par nom d'utilisateur — non bypassable par IP spoofing.
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
