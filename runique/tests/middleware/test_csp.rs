// Tests pour csp middleware

use crate::helpers::{
    assert::{assert_has_header, assert_header_eq, assert_status},
    request,
    server::build_engine,
};
use axum::{Router, middleware, routing::get};
use runique::app::staging::CspConfig;
use runique::middleware::security::csp::{
    SecurityPolicy, csp_middleware, csp_report_only_middleware, https_redirect_middleware,
    security_headers_middleware,
};
use runique::utils::aliases::AEngine;

#[test]
fn test_security_policy_default() {
    let policy = SecurityPolicy::default();
    assert!(policy.default_src.contains(&"'self'".to_string()));
    assert!(policy.script_src.contains(&"'self'".to_string()));
    assert!(policy.style_src.contains(&"'self'".to_string()));
    assert!(policy.img_src.contains(&"'self'".to_string()));
    assert!(policy.use_nonce);
}

#[test]
fn test_security_policy_strict() {
    let policy = SecurityPolicy::strict();
    assert!(policy.use_nonce);
    assert_eq!(policy.frame_ancestors, vec!["'none'".to_string()]);
}

#[test]
fn test_security_policy_permissive() {
    let policy = SecurityPolicy::permissive();
    assert!(!policy.use_nonce);
    assert!(policy.script_src.contains(&"'unsafe-eval'".to_string()));
    assert!(policy.img_src.contains(&"https:".to_string()));
}

#[test]
fn test_csp_config_default_policy() {
    // CspConfig::default() demarre avec SecurityPolicy::default()
    let csp = CspConfig::default();
    assert!(csp.get_policy().default_src.contains(&"'self'".to_string()));
    assert!(!csp.header_security_enabled());
}

#[test]
fn test_to_header_value_basic() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(None);
    assert!(header.contains("default-src"));
    assert!(header.contains("script-src"));
    assert!(header.contains("style-src"));
    assert!(header.contains("img-src"));
}

#[test]
fn test_to_header_value_with_nonce() {
    let mut policy = SecurityPolicy {
        use_nonce: true,
        ..Default::default()
    };
    policy.use_nonce = true;
    let header = policy.to_header_value(Some("abc123"));
    assert!(header.contains("'nonce-abc123'"));
}

#[test]
fn test_to_header_value_contains_all_directives() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(None);
    assert!(header.contains("font-src"));
    assert!(header.contains("connect-src"));
    assert!(header.contains("frame-ancestors"));
    assert!(header.contains("base-uri"));
    assert!(header.contains("form-action"));
}

#[test]
fn test_to_header_value_removes_unsafe_inline_with_nonce() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(Some("mynonce"));
    // Le nonce est injecté et 'unsafe-inline' est retiré
    assert!(header.contains("'nonce-mynonce'"));
    assert!(!header.contains("'unsafe-inline'"));
}

#[test]
fn test_strict_no_unsafe_inline_in_script() {
    let policy = SecurityPolicy::strict();
    let header = policy.to_header_value(Some("nonce_strict"));
    assert!(!header.contains("'unsafe-inline'"));
    assert!(header.contains("'nonce-nonce_strict'"));
}

#[test]
fn test_permissive_frame_ancestors_self() {
    let policy = SecurityPolicy::permissive();
    assert_eq!(policy.frame_ancestors, vec!["'self'".to_string()]);
    let header = policy.to_header_value(None);
    assert!(header.contains("frame-ancestors 'self'"));
}

#[test]
fn test_csp_config_custom_default_src() {
    let csp = CspConfig::default().default_src(vec!["'self'", "cdn.example.com"]);
    assert!(
        csp.get_policy()
            .default_src
            .contains(&"cdn.example.com".to_string())
    );
}

#[test]
fn test_csp_config_custom_scripts() {
    let csp = CspConfig::default().scripts(vec!["'self'", "cdn.js.com"]);
    assert!(
        csp.get_policy()
            .script_src
            .contains(&"cdn.js.com".to_string())
    );
}

#[test]
fn test_csp_config_nonce_false() {
    let csp = CspConfig::default().with_nonce(false);
    assert!(!csp.get_policy().use_nonce);
}

#[test]
fn test_csp_config_nonce_true_par_defaut() {
    let csp = CspConfig::default();
    assert!(csp.get_policy().use_nonce);
}

#[test]
fn test_csp_config_header_security() {
    let csp = CspConfig::default().with_header_security(true);
    assert!(csp.header_security_enabled());
}

#[test]
fn test_csp_config_upgrade_insecure() {
    let csp = CspConfig::default().with_upgrade_insecure(true);
    assert!(csp.get_policy().upgrade_insecure_requests);
}

#[test]
fn test_csp_config_preset_strict() {
    let csp = CspConfig::default().policy(SecurityPolicy::strict());
    assert!(csp.get_policy().use_nonce);
    assert!(csp.get_policy().upgrade_insecure_requests);
}

// ── CspConfig — builder methods manquants ───────────────────────

#[test]
fn test_csp_config_styles() {
    let csp = CspConfig::default().styles(vec!["'self'", "cdn.example.com"]);
    assert!(
        csp.get_policy()
            .style_src
            .contains(&"cdn.example.com".to_string())
    );
}

#[test]
fn test_csp_config_images() {
    let csp = CspConfig::default().images(vec!["'self'", "data:"]);
    assert!(csp.get_policy().img_src.contains(&"data:".to_string()));
}

#[test]
fn test_csp_config_fonts() {
    let csp = CspConfig::default().fonts(vec!["'self'", "https://fonts.gstatic.com"]);
    assert!(
        csp.get_policy()
            .font_src
            .contains(&"https://fonts.gstatic.com".to_string())
    );
}

#[test]
fn test_csp_config_connect() {
    let csp = CspConfig::default().connect(vec!["'self'", "https://api.example.com"]);
    assert!(
        csp.get_policy()
            .connect_src
            .contains(&"https://api.example.com".to_string())
    );
}

#[test]
fn test_csp_config_objects() {
    let csp = CspConfig::default().objects(vec!["'none'"]);
    assert!(csp.get_policy().object_src.contains(&"'none'".to_string()));
}

#[test]
fn test_csp_config_media() {
    let csp = CspConfig::default().media(vec!["'self'", "https://media.example.com"]);
    assert!(
        csp.get_policy()
            .media_src
            .contains(&"https://media.example.com".to_string())
    );
}

#[test]
fn test_csp_config_frames() {
    let csp = CspConfig::default().frames(vec!["'self'"]);
    assert!(csp.get_policy().frame_src.contains(&"'self'".to_string()));
}

#[test]
fn test_csp_config_frame_ancestors_none() {
    let csp = CspConfig::default().frame_ancestors(vec!["'none'"]);
    assert!(
        csp.get_policy()
            .frame_ancestors
            .contains(&"'none'".to_string())
    );
}

#[test]
fn test_csp_config_base_uri() {
    let csp = CspConfig::default().base_uri(vec!["'self'"]);
    assert!(csp.get_policy().base_uri.contains(&"'self'".to_string()));
}

#[test]
fn test_csp_config_form_action() {
    let csp = CspConfig::default().form_action(vec!["'self'", "https://form.example.com"]);
    assert!(
        csp.get_policy()
            .form_action
            .contains(&"https://form.example.com".to_string())
    );
}

#[test]
fn test_csp_config_preset_permissive() {
    let csp = CspConfig::default().policy(SecurityPolicy::permissive());
    assert!(!csp.get_policy().use_nonce);
    assert!(
        csp.get_policy()
            .script_src
            .contains(&"'unsafe-eval'".to_string())
    );
}

// ── to_header_value — branches restantes ─────────────────────────────────────

#[test]
fn test_to_header_value_upgrade_insecure() {
    let policy = SecurityPolicy::strict(); // upgrade_insecure_requests = true
    let header = policy.to_header_value(None);
    assert!(header.contains("upgrade-insecure-requests"));
}

#[test]
fn test_to_header_value_nonce_vide_pas_injecte() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(Some(""));
    // Nonce vide → filtré → pas injecté
    assert!(!header.contains("'nonce-'"));
}

#[test]
fn test_to_header_value_object_media_frame_src() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(None);
    assert!(header.contains("object-src"));
    assert!(header.contains("media-src"));
    assert!(header.contains("frame-src"));
}

// ── Middlewares HTTP ──────────────────────────────────────────────────────────

fn csp_app(engine: AEngine) -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(engine, csp_middleware))
}

fn csp_report_only_app(engine: AEngine) -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            engine,
            csp_report_only_middleware,
        ))
}

fn security_headers_app(engine: AEngine) -> Router {
    Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            engine,
            security_headers_middleware,
        ))
}

fn https_redirect_app(engine: AEngine) -> Router {
    Router::new()
        .route("/path", get(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            engine,
            https_redirect_middleware,
        ))
}

#[tokio::test]
async fn test_csp_middleware_ajoute_header() {
    let engine = build_engine().await;
    let resp = request::get(csp_app(engine), "/").await;
    assert_status(&resp, 200);
    assert_has_header(&resp, "content-security-policy");
}

#[tokio::test]
async fn test_csp_middleware_header_contient_default_src() {
    let engine = build_engine().await;
    let resp = request::get(csp_app(engine), "/").await;
    let csp = resp
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(csp.contains("default-src"));
}

#[tokio::test]
async fn test_csp_report_only_middleware_ajoute_header() {
    let engine = build_engine().await;
    let resp = request::get(csp_report_only_app(engine), "/").await;
    assert_status(&resp, 200);
    assert_has_header(&resp, "content-security-policy-report-only");
}

#[tokio::test]
async fn test_security_headers_middleware_ajoute_csp_et_autres() {
    let engine = build_engine().await;
    let resp = request::get(security_headers_app(engine), "/").await;
    assert_status(&resp, 200);
    assert_has_header(&resp, "content-security-policy");
    assert_has_header(&resp, "x-content-type-options");
    assert_has_header(&resp, "x-frame-options");
    assert_has_header(&resp, "strict-transport-security");
    assert_has_header(&resp, "referrer-policy");
}

#[tokio::test]
async fn test_security_headers_middleware_nonce_injecte_dans_csp() {
    let engine = build_engine().await;
    let resp = request::get(security_headers_app(engine), "/").await;
    let csp = resp
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();
    // use_nonce = true par défaut → nonce présent dans script-src
    assert!(csp.contains("nonce-"));
}

#[tokio::test]
async fn test_security_headers_middleware_x_frame_options_deny() {
    let engine = build_engine().await;
    let resp = request::get(security_headers_app(engine), "/").await;
    assert_header_eq(&resp, "x-frame-options", "DENY");
}

#[tokio::test]
async fn test_security_headers_middleware_hsts_present() {
    let engine = build_engine().await;
    let resp = request::get(security_headers_app(engine), "/").await;
    let hsts = resp
        .headers()
        .get("strict-transport-security")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(hsts.contains("max-age=31536000"));
}

#[tokio::test]
async fn test_https_redirect_disabled_par_defaut() {
    // enforce_https = false par défaut → pas de redirection
    let engine = build_engine().await;
    let resp = request::get(https_redirect_app(engine), "/path").await;
    assert_status(&resp, 200);
}

#[tokio::test]
async fn test_https_redirect_redirige_quand_actif() {
    use runique::engine::RuniqueEngine;
    use runique::middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy},
    };
    use std::sync::Arc;

    let engine = build_engine().await;
    let mut config = engine.config.clone();
    config.security.enforce_https = true;

    let engine_https = Arc::new(RuniqueEngine {
        config,
        tera: engine.tera.clone(),
        db: engine.db.clone(),
        url_registry: engine.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        session_store: std::sync::OnceLock::new(),
    });

    // Requête sans X-Forwarded-Proto: https → redirection (308 Permanent Redirect)
    let resp = request::get(https_redirect_app(engine_https), "/path").await;
    assert!(
        resp.status().is_redirection(),
        "Redirection attendue, reçu {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_https_redirect_passe_si_deja_https() {
    use axum::body::Body;
    use axum::http::Request;
    use runique::engine::RuniqueEngine;
    use runique::middleware::{
        config::MiddlewareConfig,
        security::{allowed_hosts::HostPolicy, csp::SecurityPolicy},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    let engine = build_engine().await;
    let mut config = engine.config.clone();
    config.security.enforce_https = true;

    let engine_https = Arc::new(RuniqueEngine {
        config,
        tera: engine.tera.clone(),
        db: engine.db.clone(),
        url_registry: engine.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        session_store: std::sync::OnceLock::new(),
    });

    let app = https_redirect_app(engine_https);
    let req = Request::builder()
        .uri("/path")
        .header("x-forwarded-proto", "https")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}
