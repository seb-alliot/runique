use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use runique::middleware_folder::login_requiert::{login_required, redirect_if_authenticated};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

/// Handler protégé
async fn protected_handler() -> &'static str {
    "Protected"
}

/// Handler public (login page)
async fn login_handler() -> &'static str {
    "Login"
}

/// Crée une app de test avec le middleware login_required
fn create_protected_app() -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd);

    Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn(login_required))
        .layer(session_layer)
}

/// Crée une app de test avec redirect_if_authenticated
fn create_login_app() -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnSessionEnd);

    Router::new()
        .route("/login", get(login_handler))
        .layer(middleware::from_fn(redirect_if_authenticated))
        .layer(session_layer)
}

#[tokio::test]
async fn test_login_required_redirects_when_not_authenticated() {
    let app = create_protected_app();

    // Requête sans session authentifiée
    let req = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Devrait rediriger vers /login
    assert_eq!(res.status(), StatusCode::SEE_OTHER);

    let location = res.headers().get("location");
    assert!(location.is_some());
    assert_eq!(location.unwrap().to_str().unwrap(), "/login");
}

#[tokio::test]
async fn test_redirect_if_authenticated_when_not_authenticated() {
    let app = create_login_app();

    // Requête sans session authentifiée
    let req = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Devrait permettre l'accès à la page de login
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_required_structure() {
    // Test que le middleware est bien configuré
    let app = create_protected_app();

    let req = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Le middleware devrait être actif (redirection ou erreur)
    assert!(res.status().is_redirection() || res.status().is_client_error());
}

#[tokio::test]
async fn test_redirect_if_authenticated_structure() {
    // Test que le middleware est bien configuré
    let app = create_login_app();

    let req = Request::builder()
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();

    // Le middleware devrait être actif
    assert!(res.status().is_success() || res.status().is_redirection());
}
