//! Content Security Policy Middleware: generates CSP headers with nonce per request.
use crate::context::RequestExtensions;
use crate::utils::{aliases::AEngine, csp_nonce::CspNonce};

/// Hashes of inline styles injected by htmx (embedded version: 2.0.4).
///
/// These hashes are deterministic: SHA-256 of the exact injected style value
/// (e.g., `display:none`). They do not change for a given value, but must
/// be updated if the htmx version changes and injects different values.
///
/// Reference: `runique/templates/admin/composant/list.html`
/// → `https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js`
///
/// To add a missing hash: the browser indicates it in the CSP console.
pub const HTMX_STYLE_HASHES: &[&str] = &["'sha256-bsV5JivYxvGywDAZ22EZJKBFip65Ng9xoJVLbBg7bdo='"];
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub default_src: Vec<String>,
    pub script_src: Vec<String>,
    pub style_src: Vec<String>,
    pub img_src: Vec<String>,
    pub font_src: Vec<String>,
    pub connect_src: Vec<String>,
    /// Authorized sources for embedded objects (plugins, applets)
    pub object_src: Vec<String>,
    /// Authorized sources for audio/video media
    pub media_src: Vec<String>,
    /// Authorized sources for iframes
    pub frame_src: Vec<String>,
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub upgrade_insecure_requests: bool,
    pub use_nonce: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'none'".into()],
            script_src: vec!["'self'".into()],
            // 'unsafe-inline' required for libraries like htmx that inject
            // inline styles (e.g., style="display:none") without nonce.
            style_src: vec!["'self'".into(), "'unsafe-inline'".into()],
            // Only `'self'` by default.
            // To allow inline base64 images (avatars, rich-text editors),
            // add `data:` via the env variable:
            //   RUNIQUE_POLICY_CSP_IMAGES='self',data:
            img_src: vec!["'self'".into()],
            font_src: vec!["'self'".into()],
            connect_src: vec!["'self'".into()],
            // Block all embedded objects by default
            object_src: vec!["'none'".into()],
            // Allow media from the same domain
            media_src: vec!["'self'".into()],
            // Block iframes by default
            frame_src: vec!["'none'".into()],
            frame_ancestors: vec!["'none'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            upgrade_insecure_requests: false,
            use_nonce: true,
        }
    }
}

impl SecurityPolicy {
    pub fn strict() -> Self {
        Self {
            default_src: vec!["'none'".into()],
            script_src: vec!["'self'".into()],
            style_src: vec!["'self'".into()],
            img_src: vec!["'self'".into()],
            font_src: vec!["'self'".into()],
            connect_src: vec!["'self'".into()],
            object_src: vec!["'none'".into()],
            media_src: vec!["'self'".into()],
            frame_src: vec!["'none'".into()],
            frame_ancestors: vec!["'none'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            upgrade_insecure_requests: true,
            use_nonce: true,
        }
    }
    pub fn permissive() -> Self {
        Self {
            default_src: vec!["'none'".into()],
            script_src: vec![
                "'self'".into(),
                "'unsafe-inline'".into(),
                "'unsafe-eval'".into(),
            ],
            style_src: vec!["'self'".into(), "'unsafe-inline'".into()],
            img_src: vec!["'self'".into(), "data:".into(), "https:".into()],
            font_src: vec!["'self'".into(), "data:".into()],
            connect_src: vec!["'self'".into()],
            object_src: vec!["'self'".into()],
            media_src: vec!["'self'".into(), "https:".into()],
            frame_src: vec!["'self'".into()],
            frame_ancestors: vec!["'self'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            upgrade_insecure_requests: false,
            use_nonce: false,
        }
    }

    /// Adds known htmx inline style hashes to `style_src`.
    ///
    /// Called automatically by the builder when `.with_admin()` is activated.
    /// Avoids opening `'unsafe-inline'` on `style-src` while allowing
    /// dynamically injected styles by htmx.
    pub fn merge_htmx_hashes(&mut self) {
        for hash in HTMX_STYLE_HASHES {
            let s = hash.to_string();
            if !self.style_src.contains(&s) {
                self.style_src.push(s);
            }
        }
    }
    #[must_use]
    pub fn to_header_value(&self, nonce: Option<&str>) -> String {
        let mut directives = Vec::new();

        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }

        if !self.script_src.is_empty() {
            let mut script_sources = self.script_src.clone();
            if let Some(n) = nonce.filter(|n| !n.is_empty()) {
                script_sources.push(format!("'nonce-{n}'"));
                script_sources.retain(|s| s != "'unsafe-inline'");
            }
            directives.push(format!("script-src {}", script_sources.join(" ")));
        }

        if !self.style_src.is_empty() {
            let mut style_sources = self.style_src.clone();
            if let Some(n) = nonce.filter(|n| !n.is_empty()) {
                style_sources.push(format!("'nonce-{n}'"));
                style_sources.retain(|s| s != "'unsafe-inline'" && s != "'unsafe-hashes'");
                // ↑ Also remove 'unsafe-hashes' when nonce is present
            }
            directives.push(format!("style-src {}", style_sources.join(" ")));
        }

        if !self.img_src.is_empty() {
            directives.push(format!("img-src {}", self.img_src.join(" ")));
        }
        if !self.font_src.is_empty() {
            directives.push(format!("font-src {}", self.font_src.join(" ")));
        }
        if !self.connect_src.is_empty() {
            directives.push(format!("connect-src {}", self.connect_src.join(" ")));
        }
        if !self.object_src.is_empty() {
            directives.push(format!("object-src {}", self.object_src.join(" ")));
        }
        if !self.media_src.is_empty() {
            directives.push(format!("media-src {}", self.media_src.join(" ")));
        }
        if !self.frame_src.is_empty() {
            directives.push(format!("frame-src {}", self.frame_src.join(" ")));
        }
        if !self.frame_ancestors.is_empty() {
            directives.push(format!(
                "frame-ancestors {}",
                self.frame_ancestors.join(" ")
            ));
        }
        if !self.base_uri.is_empty() {
            directives.push(format!("base-uri {}", self.base_uri.join(" ")));
        }
        if !self.form_action.is_empty() {
            directives.push(format!("form-action {}", self.form_action.join(" ")));
        }

        if self.upgrade_insecure_requests {
            directives.push("upgrade-insecure-requests".to_string());
        }

        directives.join("; ")
    }
}

/// Standard CSP Middleware
pub async fn csp_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut response: axum::http::Response<Body> = next.run(req).await;

    let csp_value = engine.security_csp.to_header_value(None);
    if let Ok(header) = HeaderValue::from_str(&csp_value) {
        response
            .headers_mut()
            .insert(axum::http::header::CONTENT_SECURITY_POLICY, header);
    }

    response
}

/// Global security middleware (CSP + miscellaneous headers)
pub async fn security_headers_middleware(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Generate a unique nonce for this request
    let nonce = CspNonce::generate();

    // Injection via centralized structure
    let extensions = RequestExtensions::new().with_csp_nonce(nonce.clone());

    extensions.inject_request(&mut req);

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Use the nonce to build the CSP
    let csp_value = engine.security_csp.to_header_value(Some(nonce.as_str()));
    if let Ok(header) = HeaderValue::from_str(&csp_value) {
        headers.insert(axum::http::header::CONTENT_SECURITY_POLICY, header);
    }

    // Other security headers
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    headers.insert(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    headers.insert(
        "cross-origin-embedder-policy",
        HeaderValue::from_static("require-corp"),
    );

    headers.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin"),
    );

    headers.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("same-origin"),
    );

    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    response
}

/// HTTPS redirection middleware
pub async fn https_redirect_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Check if enforce_https is enabled
    if !engine.config.security.enforce_https {
        return next.run(req).await;
    }

    // Check if the request is already in HTTPS
    // Behind a proxy, check X-Forwarded-Proto
    let is_https = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("https"));

    if is_https {
        return next.run(req).await;
    }

    // Build the HTTPS URL
    let host = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");

    let uri = req.uri();
    let https_url = format!(
        "https://{}{}",
        host,
        uri.path_and_query().map_or("", |pq| pq.as_str())
    );

    // Redirect with 301
    Redirect::permanent(&https_url).into_response()
}
