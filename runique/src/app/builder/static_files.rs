//! The static files and the media, served outside the middleware stack with
//! their own headers.
use super::RuniqueAppBuilder;
use crate::config::RuniqueConfig;
use axum::{
    Router,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::borrow::Cow;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

/// Policy of every media response: no script, no plugin, whatever the file
/// claims to be (an uploaded `.html` or `.xhtml` is otherwise a page of the site).
const MEDIA_CSP: &str = "script-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'";

/// 404 for any path segment starting with a dot, decoded first (`%2E` is a
/// dot to `ServeDir`): uploads wait in `.staging-*` folders under the media
/// root until their form is checked, and must not be reachable meanwhile.
async fn hide_dot_segments(req: Request, next: Next) -> Response {
    use axum::response::IntoResponse;
    let hidden = req.uri().path().split('/').any(|segment| {
        urlencoding::decode(segment).map_or(true, |decoded| decoded.starts_with('.'))
    });
    if hidden {
        return StatusCode::NOT_FOUND.into_response();
    }
    next.run(req).await
}

impl<P> RuniqueAppBuilder<P> {
    pub(super) fn attach_static_files(
        mut router: Router,
        config: &RuniqueConfig,
        static_cache: Cow<'static, str>,
        media_cache: Cow<'static, str>,
    ) -> Router {
        // HSTS is host-scoped: once any page emits it (via the CSP middleware,
        // gated + configured by `SecurityConfig::hsts_header_value`), the browser
        // applies it to the whole host, static assets included. So static file
        // responses only carry the per-asset headers here — no HSTS to keep
        // config as the single source of truth (never a hardcoded preload).
        let security_headers = || {
            tower::ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::if_not_present(
                    HeaderName::from_static("x-content-type-options"),
                    HeaderValue::from_static("nosniff"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    HeaderName::from_static("x-frame-options"),
                    HeaderValue::from_static("DENY"),
                ))
                .layer(SetResponseHeaderLayer::if_not_present(
                    HeaderName::from_static("referrer-policy"),
                    HeaderValue::from_static("strict-origin-when-cross-origin"),
                ))
        };

        // In development the `?v=` token stays the same while a file is edited:
        // the browser must ask again instead of keeping its `immutable` copy.
        let static_cache = if config.dev {
            "no-cache"
        } else {
            static_cache.as_ref()
        };
        // Value already validated in `StaticStaging::validate()` (build Step 1),
        // run before this function — the header is well-formed by construction.
        let static_headers = security_headers().layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_str(static_cache).expect("validated in StaticStaging::validate()"),
        ));

        // Media are files users uploaded: served from the app's own origin, an
        // uploaded page must never run a script there. The middleware stack
        // (and its CSP) doesn't cover these routes, so the policy is set here.
        let media_headers = security_headers()
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static(MEDIA_CSP),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static("cache-control"),
                HeaderValue::from_str(&media_cache)
                    .expect("validated in StaticStaging::validate()"),
            ))
            .layer(axum::middleware::from_fn(hide_dot_segments));

        router = router
            .nest_service(
                &config.static_files.static_url,
                static_headers
                    .clone()
                    .service(ServeDir::new(&config.static_files.staticfiles_dirs)),
            )
            .nest_service(
                &config.static_files.media_url,
                media_headers.service(ServeDir::new(&config.static_files.media_root)),
            );

        if !config.static_files.static_runique_url.is_empty() {
            router = router.nest_service(
                &config.static_files.static_runique_url,
                static_headers.service(ServeDir::new(&config.static_files.static_runique_path)),
            );
        }

        if let Some(level) = crate::log::log_level!(builder.statics) {
            crate::runique_log!(
                level,
                static_url = %config.static_files.static_url,
                static_dir = %config.static_files.staticfiles_dirs,
                media_url = %config.static_files.media_url,
                media_dir = %config.static_files.media_root,
                "{}", crate::translation::t(runique_i18n::keys::build::log::STATIC_FILES_ATTACHED));
        }

        router
    }
}
