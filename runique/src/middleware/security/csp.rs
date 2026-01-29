use crate::aliases::AEngine;
use crate::context::RequestExtensions;
use crate::utils::csp_nonce::CspNonce;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
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
    pub frame_ancestors: Vec<String>,
    pub base_uri: Vec<String>,
    pub form_action: Vec<String>,
    pub use_nonce: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".into()],
            script_src: vec!["'self'".into(), "'unsafe-inline'".into()],
            style_src: vec!["'self'".into(), "'unsafe-inline'".into()],
            img_src: vec!["'self'".into(), "data:".into()],
            font_src: vec!["'self'".into()],
            connect_src: vec!["'self'".into()],
            frame_ancestors: vec!["'none'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            use_nonce: false,
        }
    }
}

impl SecurityPolicy {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        let get_list = |key: &str| -> Option<Vec<String>> {
            std::env::var(key).ok().map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
        };

        // On utilise les nouveaux noms explicites
        if let Some(list) = get_list("RUNIQUE_POLICY_CSP_DEFAULT") {
            config.default_src = list;
        }
        if let Some(list) = get_list("RUNIQUE_POLICY_CSP_SCRIPTS") {
            config.script_src = list;
        }
        if let Some(list) = get_list("RUNIQUE_POLICY_CSP_STYLES") {
            config.style_src = list;
        }
        if let Some(list) = get_list("RUNIQUE_POLICY_CSP_IMAGES") {
            config.img_src = list;
        }
        if let Some(list) = get_list("RUNIQUE_POLICY_CSP_FONTS") {
            config.font_src = list;
        }

        config.use_nonce = std::env::var("RUNIQUE_POLICY_CSP_STRICT_NONCE")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);

        config
    }

    pub fn strict() -> Self {
        Self {
            default_src: vec!["'self'".into()],
            script_src: vec!["'self'".into()],
            style_src: vec!["'self'".into()],
            img_src: vec!["'self'".into()],
            font_src: vec!["'self'".into()],
            connect_src: vec!["'self'".into()],
            frame_ancestors: vec!["'none'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            use_nonce: true,
        }
    }

    pub fn permissive() -> Self {
        Self {
            default_src: vec!["'self'".into()],
            script_src: vec![
                "'self'".into(),
                "'unsafe-inline'".into(),
                "'unsafe-eval'".into(),
            ],
            style_src: vec!["'self'".into(), "'unsafe-inline'".into()],
            img_src: vec!["'self'".into(), "data:".into(), "https:".into()],
            font_src: vec!["'self'".into(), "data:".into()],
            connect_src: vec!["'self'".into()],
            frame_ancestors: vec!["'self'".into()],
            base_uri: vec!["'self'".into()],
            form_action: vec!["'self'".into()],
            use_nonce: false,
        }
    }

    pub fn to_header_value(&self, nonce: Option<&str>) -> String {
        let mut directives = Vec::new();

        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }

        if !self.script_src.is_empty() {
            let mut script_sources = self.script_src.clone();
            script_sources.push(format!("'nonce-{}'", nonce.unwrap_or("")));
            script_sources.retain(|s| s != "'unsafe-inline'");
            directives.push(format!("script-src {}", script_sources.join(" ")));
        }

        if !self.style_src.is_empty() {
            let mut style_sources = self.style_src.clone();
            style_sources.push(format!("'nonce-{}'", nonce.unwrap_or("")));
            style_sources.push(format!("'nonce-{}'", nonce.unwrap_or("")));
            style_sources.retain(|s| s != "'unsafe-inline'");
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

        directives.join("; ")
    }
}

/// Middleware CSP standard
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

/// Middleware CSP report-only
pub async fn csp_report_only_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;

    let csp_value = engine.security_csp.to_header_value(None);
    if let Ok(header) = HeaderValue::from_str(&csp_value) {
        response.headers_mut().insert(
            axum::http::header::CONTENT_SECURITY_POLICY_REPORT_ONLY,
            header,
        );
    }

    response
}

/// Middleware global de sécurité (CSP + headers divers)
pub async fn security_headers_middleware(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Générer un nonce unique pour cette requête
    let nonce = CspNonce::generate();

    // Injection via la structure centralisée
    let extensions = RequestExtensions::new().with_csp_nonce(nonce.clone());

    extensions.inject_request(&mut req);

    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Utiliser le nonce pour construire la CSP
    let csp_value = engine.security_csp.to_header_value(Some(nonce.as_str()));
    if let Ok(header) = HeaderValue::from_str(&csp_value) {
        headers.insert(axum::http::header::CONTENT_SECURITY_POLICY, header);
    }

    // Autres headers de sécurité
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

    response
}
