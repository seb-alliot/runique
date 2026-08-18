// Tests pour la génération automatique de /robots.txt quand l'admin est activé

use axum::Router;
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::auth::session::{AdminAuth, AdminLoginResult};
use runique::config::RuniqueConfig;
use sea_orm::{Database, DatabaseConnection};
use serial_test::serial;

// `#[serial]` : chaque test construit une app admin complete, qui pousse ses
// routes nommees dans le registre global `PENDING_URLS` (partage avec
// `test_admin_prefix.rs`, egalement `#[serial]`). Voir `register_url.rs`.

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

/// `mount` est le segment de `.prefix()` ; le chemin de l'admin reste le
/// défaut `/admin`, donc l'admin est servi sous `{mount}/admin`.
async fn build_app_with_admin(mount: &str) -> axum::Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| a.prefix(mount).auth(MockAdminAuth))
        .build()
        .await
        .unwrap();

    app.router
}

async fn build_app_with_admin_no_robots(mount: &str) -> axum::Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| a.prefix(mount).auth(MockAdminAuth).no_robots_txt())
        .build()
        .await
        .unwrap();

    app.router
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// robots.txt est servi automatiquement quand l'admin est activé.
#[tokio::test]
#[serial]
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
}

/// Le préfixe admin ne doit JAMAIS apparaître dans robots.txt : le fichier est
/// public, et un préfixe personnalisé est souvent choisi pour rester discret.
/// La désindexation passe par l'en-tête `X-Robots-Tag` (test suivant).
#[tokio::test]
#[serial]
async fn test_robots_txt_ne_divulgue_pas_le_prefix() {
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
    assert!(
        !body_str.contains("backoffice"),
        "robots.txt divulgue le préfixe admin : {body_str}"
    );
    assert!(!body_str.contains("Disallow"));
}

/// La page de login admin porte `X-Robots-Tag: noindex` — c'est la seule page
/// admin atteignable sans compte, donc la seule réellement indexable. Elle est
/// hors du middleware `admin_required`, d'où l'en-tête posé au-dessus.
#[tokio::test]
#[serial]
async fn test_x_robots_tag_sur_login_admin() {
    use axum::http::Request;
    use tower::ServiceExt;

    let app = build_app_with_admin("/backoffice").await;

    let req = Request::builder()
        .uri("/backoffice/admin/login")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let header = resp
        .headers()
        .get("x-robots-tag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    assert!(
        header.contains("noindex"),
        "en-tête X-Robots-Tag absent ou sans noindex : '{header}'"
    );
}

/// .no_robots_txt() désactive la route — le contenu robots.txt n'est pas servi.
#[tokio::test]
#[serial]
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
