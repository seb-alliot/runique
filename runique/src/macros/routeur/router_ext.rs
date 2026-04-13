//! `RouterExt` — extension d'Axum `Router` pour attacher un rate limiter ou un guard de login à une route.
use std::sync::Arc;

use axum::{Router, routing::MethodRouter};

use crate::auth::guard::login_required_middleware;
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
    /// Ajoute une route protégée par `login_required` — redirige vers `redirect_url` si non authentifié.
    ///
    /// # Exemple
    /// ```rust,ignore
    /// urlpatterns! { ... }
    ///     .login_required("/profil", "profil", view!(profil), "/login")
    /// ```
    fn login_required(
        self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        redirect_url: impl Into<String>,
    ) -> Self;

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
    fn login_required(
        self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        redirect_url: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let name = name.into();
        let redirect = Arc::new(redirect_url.into());
        register_pending(&name, &path);
        let protected =
            Router::new()
                .route(&path, handler)
                .route_layer(axum::middleware::from_fn_with_state(
                    redirect,
                    login_required_middleware,
                ));
        self.merge(protected)
    }

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
