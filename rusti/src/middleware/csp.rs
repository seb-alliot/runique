use tower_sessions::Session;
use axum::{
    response::Response,
    middleware::Next,
    http::HeaderValue,
    extract::Request,
};
use std::sync::Arc;
use crate::settings::Settings;
use crate::utils::generate_token;

pub const NONCE_KEY: &str = "csp_nonce";

/// Configuration de la Content Security Policy
#[derive(Clone, Debug)]
pub struct CspConfig {
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

impl Default for CspConfig {
    /// Configuration par défaut
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            use_nonce: false,
        }
    }
}

impl CspConfig {
    /// Configuration stricte => production
    pub fn strict() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string()],
            img_src: vec!["'self'".to_string()],
            font_src: vec!["'self'".to_string()],
            connect_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'none'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            use_nonce: true,
        }
    }

    /// Configuration permissive => dév
    pub fn permissive() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string(), "'unsafe-eval'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()],
            font_src: vec!["'self'".to_string(), "data:".to_string()],
            connect_src: vec!["'self'".to_string()],
            frame_ancestors: vec!["'self'".to_string()],
            base_uri: vec!["'self'".to_string()],
            form_action: vec!["'self'".to_string()],
            use_nonce: false,
        }
    }

    /// Génère la valeur de l'en-tête CSP
    fn to_header_value(&self, nonce: Option<&str>) -> String {
        let mut directives = Vec::new();

        if !self.default_src.is_empty() {
            directives.push(format!("default-src {}", self.default_src.join(" ")));
        }

        // Script-src avec nonce si activé
        if !self.script_src.is_empty() {
            let mut script_sources = self.script_src.clone();
            if let Some(n) = nonce {
                script_sources.push(format!("'nonce-{}'", n));
                // Retirer 'unsafe-inline' si nonce présent (bonne pratique)
                script_sources.retain(|s| s != "'unsafe-inline'");
            }
            directives.push(format!("script-src {}", script_sources.join(" ")));
        }

        // Style-src avec nonce si activé
        if !self.style_src.is_empty() {
            let mut style_sources = self.style_src.clone();
            if let Some(n) = nonce {
                style_sources.push(format!("'nonce-{}'", n));
                // Retirer 'unsafe-inline' si nonce présent
                style_sources.retain(|s| s != "'unsafe-inline'");
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
        if !self.frame_ancestors.is_empty() {
            directives.push(format!("frame-ancestors {}", self.frame_ancestors.join(" ")));
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

/// Middleware CSP (Content Security Policy)
///
/// # Exemple
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn index() -> &'static str { "Hello" }
/// use rusti::middleware::csp::{csp_middleware, CspConfig};
///
/// // Préciser Router<()> règle l'erreur E0283
/// let app: Router = Router::new()
///     .route("/", get(index))
///     .layer(axum::middleware::from_fn_with_state(
///         CspConfig::default(),
///         csp_middleware
///     ));
/// ```
pub async fn csp_middleware(
    axum::extract::State(config): axum::extract::State<CspConfig>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    // Ajouter l'en-tête CSP
    let csp_value = config.to_header_value(None);
    if let Ok(header_value) = HeaderValue::from_str(&csp_value) {
        response.headers_mut().insert(
            axum::http::header::CONTENT_SECURITY_POLICY,
            header_value,
        );
    }

    response
}

/// Middleware pour ajouter tous les en-têtes de sécurité
///
/// Inclut :
/// - Content-Security-Policy
/// - X-Content-Type-Options: nosniff
/// - X-Frame-Options: DENY
/// - X-XSS-Protection: 1; mode=block
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - Permissions-Policy
///
/// # Exemple
///
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn index() -> &'static str { "Hello" }
/// use rusti::middleware::csp::{csp_middleware, CspConfig};
///
/// // Préciser Router<()> règle l'erreur E0283
/// let app: Router = Router::new()
///     .route("/", get(index))
///     .layer(axum::middleware::from_fn_with_state(
///         CspConfig::default(),
///         csp_middleware
///     ));
/// ```
pub async fn security_headers_middleware(
    axum::extract::State(csp_config): axum::extract::State<CspConfig>,
    mut request: Request,
    next: Next,
) -> Response {
    let config = match request.extensions().get::<Arc<Settings>>().cloned() {
        Some(c) => c,
        None => return next.run(request).await,
    };
    let session = match request.extensions().get::<Session>() {
        Some(s) => s,
        None => return next.run(request).await,
    };
    let nonce = if csp_config.use_nonce {
        Some(generate_token(&config.server.secret_key, &session.id().unwrap_or_default().to_string()))
    } else {
        None
    };

    if let Some(ref n) = nonce {
        request.extensions_mut().insert(n.clone());
    }

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Content Security Policy avec nonce
    let csp_value = csp_config.to_header_value(nonce.as_deref());
    if let Ok(header_value) = HeaderValue::from_str(&csp_value) {
        headers.insert(
            axum::http::header::CONTENT_SECURITY_POLICY,
            header_value,
        );
    }

    // X-Content-Type-Options
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // X-Frame-Options
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions-Policy
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    response
}

/// Middleware CSP en mode report-only => pour tester sans bloquer
///
/// # Exemple
///
/// ```rust
/// # use axum::{Router, routing::get};
/// # async fn index() -> &'static str { "Hello" }
/// use rusti::middleware::csp::{csp_middleware, CspConfig};
///
/// // Préciser Router<()> règle l'erreur E0283
/// let app: Router = Router::new()
///     .route("/", get(index))
///     .layer(axum::middleware::from_fn_with_state(
///         CspConfig::default(),
///         csp_middleware
///     ));
/// ```
pub async fn csp_report_only_middleware(
    axum::extract::State(config): axum::extract::State<CspConfig>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    // Ajouter l'en-tête CSP en mode report-only
    let csp_value = config.to_header_value(None);
    if let Ok(header_value) = HeaderValue::from_str(&csp_value) {
        response.headers_mut().insert(
            axum::http::header::CONTENT_SECURITY_POLICY_REPORT_ONLY,
            header_value,
        );
    }

    response
}