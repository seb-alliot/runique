use std::sync::Arc;

use axum::{Router, routing::MethodRouter};

use crate::macros::routeur::register_url::register_pending;
use crate::middleware::rate_limit::{RateLimiter, rate_limit_middleware};

/// Extension de `Router` pour ajouter des routes avec rate limiting de façon fluente.
///
/// # Exemple — route unique
/// ```rust,ignore
/// urlpatterns! { ... }
///     .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60)
/// ```
///
/// # Exemple — plusieurs routes, compteur partagé
/// ```rust,ignore
/// urlpatterns! { ... }
///     .rate_limit_many(5, 60, vec![
///         ("/upload-image".into(), "upload_image".into(), view!(upload_image_submit)),
///         ("/inscription".into(),  "inscription".into(),  view!(soumission_inscription)),
///     ])
/// ```
pub trait RouterExt {
    /// Ajoute une route protégée par un rate limiter.
    fn rate_limit(
        self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        max_requests: u32,
        retry_after: u64,
    ) -> Self;

    /// Ajoute plusieurs routes partageant le même rate limiter (compteur commun).
    fn rate_limit_many(
        self,
        max_requests: u32,
        retry_after: u64,
        routes: Vec<(String, String, MethodRouter)>,
    ) -> Self;
}

impl RouterExt for Router {
    fn rate_limit(
        self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        max_requests: u32,
        retry_after: u64,
    ) -> Self {
        self.rate_limit_many(
            max_requests,
            retry_after,
            vec![(path.into(), name.into(), handler)],
        )
    }

    fn rate_limit_many(
        self,
        max_requests: u32,
        retry_after: u64,
        routes: Vec<(String, String, MethodRouter)>,
    ) -> Self {
        let limiter = Arc::new(
            RateLimiter::new()
                .max_requests(max_requests)
                .retry_after(retry_after),
        );
        limiter.spawn_cleanup(tokio::time::Duration::from_secs(retry_after));
        let mut r = self;
        for (path, name, handler) in routes {
            register_pending(&name, &path);
            let limited = Router::new().route(&path, handler).route_layer(
                axum::middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware),
            );
            r = r.merge(limited);
        }
        r
    }
}
