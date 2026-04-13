// Tests pour la génération automatique de /robots.txt quand l'admin est activé

use axum::Router;
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::auth::session::{AdminAuth, AdminLoginResult};
use runique::config::RuniqueConfig;
use sea_orm::{Database, DatabaseConnection};

// ── Mock AdminAuth ────────────────────────────────────────────────────────────

struct MockAdminAuth;

#[async_trait::async_trait]
impl AdminAuth for MockAdminAuth {
    async fn authenticate(
        &self,
        _username: &str,
        _password: &str,
        _db: &DatabaseConnection,
    ) -> Option<AdminLoginResult> {
        None
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

async fn build_app_with_admin(prefix: &str) -> axum::Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let config = RuniqueConfig::from_env();

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| a.prefix(prefix).auth(MockAdminAuth))
        .build()
        .await
        .unwrap();

    app.router
}

async fn build_app_with_admin_no_robots(prefix: &str) -> axum::Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let config = RuniqueConfig::from_env();

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| a.prefix(prefix).auth(MockAdminAuth).no_robots_txt())
        .build()
        .await
        .unwrap();

    app.router
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// robots.txt est servi automatiquement quand l'admin est activé.
#[tokio::test]
async fn test_robots_txt_present_quand_admin_active() {
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let app = build_app_with_admin("/admin").await;

    let req = Request::builder()
        .uri("/robots.txt")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("User-agent: *"));
    assert!(body_str.contains("Disallow: /admin/"));
}

/// Le préfixe configuré est respecté dans le Disallow.
#[tokio::test]
async fn test_robots_txt_respecte_prefix_custom() {
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let app = build_app_with_admin("/backoffice").await;

    let req = Request::builder()
        .uri("/robots.txt")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert!(body_str.contains("Disallow: /backoffice/"));
}

/// .no_robots_txt() désactive la route — le contenu robots.txt n'est pas servi.
#[tokio::test]
async fn test_robots_txt_absent_avec_no_robots_txt() {
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let app = build_app_with_admin_no_robots("/admin").await;

    let req = Request::builder()
        .uri("/robots.txt")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    // La route n'est pas enregistrée — pas de 200 avec contenu robots.txt
    assert_ne!(resp.status(), StatusCode::OK);
}
