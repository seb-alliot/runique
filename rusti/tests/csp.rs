use axum::extract::Extension;
use axum::{
    body::Body,
    http::{header, Request},
    middleware,
    routing::get,
    Router,
};
use rusti::{
    middleware::csp::{security_headers_middleware, CspConfig},
    Settings,
};
use std::sync::Arc;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

/// Handler de test simple
async fn test_handler() -> &'static str {
    "OK"
}

/// Crée une app de test avec le middleware CSP
fn create_test_app(csp_config: CspConfig) -> Router {
    let settings = Arc::new(Settings::default_values());
    let store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd);

    Router::new()
        .route("/", get(test_handler))
        .layer(middleware::from_fn_with_state(
            csp_config,
            security_headers_middleware,
        ))
        .layer(Extension(settings))
        .layer(session_layer)
}

#[tokio::test]
async fn test_csp_headers_present() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Vérifier que les headers de sécurité sont présents
    assert!(res.headers().contains_key(header::CONTENT_SECURITY_POLICY));
    assert!(res.headers().contains_key(header::X_CONTENT_TYPE_OPTIONS));
    assert!(res.headers().contains_key(header::X_FRAME_OPTIONS));
    assert!(res.headers().contains_key(header::REFERRER_POLICY));
}

#[tokio::test]
async fn test_csp_x_content_type_options() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    let x_content_type = res.headers().get(header::X_CONTENT_TYPE_OPTIONS);
    assert!(x_content_type.is_some());
    assert_eq!(x_content_type.unwrap().to_str().unwrap(), "nosniff");
}

#[tokio::test]
async fn test_csp_x_frame_options() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    let x_frame_options = res.headers().get(header::X_FRAME_OPTIONS);
    assert!(x_frame_options.is_some());
    assert_eq!(x_frame_options.unwrap().to_str().unwrap(), "DENY");
}

#[tokio::test]
async fn test_csp_referrer_policy() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    let referrer_policy = res.headers().get(header::REFERRER_POLICY);
    assert!(referrer_policy.is_some());
    assert_eq!(
        referrer_policy.unwrap().to_str().unwrap(),
        "strict-origin-when-cross-origin"
    );
}

#[tokio::test]
async fn test_csp_x_xss_protection() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    // X-XSS-Protection est un header personnalisé
    let x_xss_protection = res.headers().get("x-xss-protection");
    assert!(x_xss_protection.is_some());
    assert_eq!(x_xss_protection.unwrap().to_str().unwrap(), "1; mode=block");
}

#[tokio::test]
async fn test_csp_permissions_policy() {
    let csp_config = CspConfig::strict();
    let app = create_test_app(csp_config);

    let req = Request::builder().uri("/").body(Body::empty()).unwrap();

    let res = app.oneshot(req).await.unwrap();

    let permissions_policy = res.headers().get("permissions-policy");
    assert!(permissions_policy.is_some());
    let policy = permissions_policy.unwrap().to_str().unwrap();
    assert!(policy.contains("geolocation=()"));
    assert!(policy.contains("microphone=()"));
    assert!(policy.contains("camera=()"));
}
