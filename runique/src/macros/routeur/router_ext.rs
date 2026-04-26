//! `RouterExt` — Axum `Router` extension to attach a rate limiter or a login guard to a route.
use std::sync::Arc;

use axum::{Router, routing::MethodRouter};

use crate::auth::guard::login_required_middleware;
use crate::macros::routeur::register_url::register_pending;
use crate::middleware::rate_limit::{RateLimiter, rate_limit_middleware};

/// `RouterExt` extension to add routes with rate limiting in a fluent way.
///
/// # Example — single route
/// ```rust,ignore
/// urlpatterns! { ... }
///     .rate_limit("/upload-image", "upload_image", view!(upload_image_submit), 5, 60)
/// ```
///
/// # Example — multiple routes, shared counter
/// ```rust,ignore
/// urlpatterns! { ... }
///     .rate_limit_many(5, 60, vec![
///         ("/upload-image".into(), "upload_image".into(), view!(upload_image_submit)),
///         ("/inscription".into(),  "inscription".into(),  view!(registration_submission)),
///     ])
/// ```
pub trait RouterExt {
    /// Adds a route protected by `login_required` — redirects to `redirect_url` if not authenticated.
    ///
    /// # Example
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

    /// Adds a route protected by a rate limiter.
    fn rate_limit(
        self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        max_requests: u32,
        retry_after: u64,
    ) -> Self;

    /// Adds multiple routes sharing the same rate limiter (common counter).
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
