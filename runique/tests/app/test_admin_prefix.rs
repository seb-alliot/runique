// `.prefix()` et `.path()` decrivent deux choses independantes :
//   .path("/site-admin")   -> ou vit l'admin
//   .prefix("secret")      -> ce qui est monte devant
// L'URL publique est la composition des deux, quel que soit l'ordre d'appel.
// Avant, les deux ecrivaient dans `config.prefix` : le dernier appele gagnait,
// et l'ordre changeait silencieusement le comportement.

use axum::Router;
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;

// `#[serial]` sur tous les tests de ce fichier : chaque test construit une app
// admin complete, ce qui pousse ses routes nommees (admin_login, admin_dashboard...)
// dans le registre global `PENDING_URLS` avant de les drainer dans son propre
// engine via `add_urls()`. Ce registre est partage par tout le process de test —
// deux builds concurrents (y compris avec `test_robots_txt.rs`, egalement `#[serial]`)
// se volent mutuellement leurs entrees. Voir `register_url.rs`.

/// Construit l'app et rend l'engine, pour interroger le registre d'URLs nommees.
async fn build_app(mount: &str, admin_path: &str) -> runique::app::RuniqueApp {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.dev = true;

    RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .with_admin(|a| a.prefix(mount).path(admin_path))
        .build()
        .await
        .unwrap()
}

/// `prefix_first` inverse l'ordre des deux appels dans la chaine du builder.
async fn build(mount: &str, admin_path: &str, prefix_first: bool) -> Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.dev = true;

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .with_admin(|a| {
            if prefix_first {
                a.prefix(mount).path(admin_path)
            } else {
                a.path(admin_path).prefix(mount)
            }
        })
        .build()
        .await
        .unwrap();

    app.router
}

async fn build_with_declarative_admin_route() -> Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.dev = true;

    let routes = runique::urlpatterns! {
        "/public" => get(|| async { "public" }), name = "public_test",
        admin "/custom/{id}" => get(|| async { "private" }),
            name = "custom_admin_test",
            for_model = runique::auth::user::Entity,
            overrides = runique::admin::AdminView::Standalone,
    };

    RuniqueApp::builder(config)
        .with_database(db)
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .with_admin(|a| a.path("/admin"))
        .build()
        .await
        .unwrap()
        .router
}

async fn status_of(app: Router, uri: &str) -> axum::http::StatusCode {
    use axum::http::Request;
    use tower::ServiceExt;

    let req = Request::builder()
        .uri(uri)
        .body(axum::body::Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap().status()
}

/// A declarative admin route keeps its literal path, but inherits the admin
/// authentication boundary. Its responses, login and admin fallback are no-store.
#[tokio::test]
#[serial]
async fn test_declarative_admin_route_is_protected_and_not_cached() {
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    let app = build_with_declarative_admin_route().await;

    let public = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/public")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public.status(), StatusCode::OK);
    assert!(
        public.headers().get(header::CACHE_CONTROL).is_none(),
        "public route should not inherit the admin no-store policy"
    );

    let custom_admin = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/custom/42")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(custom_admin.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        custom_admin
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/"),
        "an anonymous visitor must not be shown the admin prefix"
    );
    assert_eq!(
        custom_admin
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let prefixed_custom_admin = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/custom/42")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        prefixed_custom_admin.status(),
        StatusCode::NOT_FOUND,
        "the declared route must keep its literal path, without an admin prefix"
    );

    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/login")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    assert_eq!(
        login
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    // The admin's own pages still send an anonymous visitor to their login.
    let dashboard = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(dashboard.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        dashboard
            .headers()
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok()),
        Some("/admin/login")
    );

    let admin_not_found = app
        .oneshot(
            Request::builder()
                .uri("/admin/not-a-route")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_not_found.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        admin_not_found
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
}

/// Le prefixe s'ajoute devant le chemin de l'admin au lieu de le remplacer.
#[tokio::test]
#[serial]
async fn test_prefix_se_compose_avec_le_chemin_admin() {
    let app = build("secret", "/site-admin", true).await;
    assert_eq!(
        status_of(app, "/secret/site-admin/login").await,
        axum::http::StatusCode::OK
    );
}

/// Le chemin nu ne repond plus : l'admin n'est servi que derriere le prefixe.
#[tokio::test]
#[serial]
async fn test_chemin_admin_seul_absent_quand_prefixe() {
    let app = build("secret", "/site-admin", true).await;
    assert_eq!(
        status_of(app, "/site-admin/login").await,
        axum::http::StatusCode::NOT_FOUND
    );
}

/// Le coeur de la regression : les deux ordres donnent la meme URL.
#[tokio::test]
#[serial]
async fn test_ordre_des_appels_sans_effet() {
    let prefix_first = build("secret", "/site-admin", true).await;
    let routes_first = build("secret", "/site-admin", false).await;

    assert_eq!(
        status_of(prefix_first, "/secret/site-admin/login").await,
        status_of(routes_first, "/secret/site-admin/login").await,
    );
}

/// Sans `.prefix()`, les URLs restent exactement celles de `path()`.
#[tokio::test]
#[serial]
async fn test_sans_prefixe_urls_inchangees() {
    let app = build("", "/site-admin", true).await;
    assert_eq!(
        status_of(app, "/site-admin/login").await,
        axum::http::StatusCode::OK
    );
}

/// Les URLs nommees doivent pointer sur le chemin REELLEMENT servi.
///
/// Piege : les routes builtin (login, dashboard, historique) enregistrent leur
/// nom via `urlpatterns!` avec le chemin litteral qui leur est donne. Si on les
/// monte via `nest()`, l'URL servie bouge mais le nom enregistre reste l'ancien,
/// et `{% url "admin_login" %}` renvoie vers une page qui n'existe plus.
#[tokio::test]
#[serial]
async fn test_url_nommee_admin_login_suit_le_prefixe() {
    use runique::macros::reverse;

    let app = build_app("secret", "/site-admin").await;
    let url = reverse(&app.engine, "admin_login").expect("admin_login non enregistre");

    assert_eq!(url, "/secret/site-admin/login");
}

/// Idem pour le tableau de bord — c'est la cible des redirections apres login.
#[tokio::test]
#[serial]
async fn test_url_nommee_dashboard_suit_le_prefixe() {
    use runique::macros::reverse;

    let app = build_app("secret", "/site-admin").await;
    let url = reverse(&app.engine, "admin_dashboard").expect("admin_dashboard non enregistre");

    assert!(
        url.starts_with("/secret/site-admin"),
        "url nommee hors du prefixe : {url}"
    );
}

/// Les slashs sont optionnels : `secret`, `/secret` et `/secret/` sont equivalents.
#[tokio::test]
#[serial]
async fn test_slashs_optionnels_dans_le_prefixe() {
    for mount in ["secret", "/secret", "/secret/"] {
        let app = build(mount, "/site-admin", true).await;
        assert_eq!(
            status_of(app, "/secret/site-admin/login").await,
            axum::http::StatusCode::OK,
            "prefixe '{mount}' non normalise"
        );
    }
}

/// L'URL nommée `admin` (base publique) suit aussi le préfixe — écrit à partir
/// des survivants cargo-mutants (2026-10-02).
#[tokio::test]
#[serial]
async fn test_url_nommee_admin_est_la_base_publique() {
    use runique::macros::reverse;

    let app = build_app("secret", "/site-admin").await;
    assert_eq!(
        reverse(&app.engine, "admin").as_deref(),
        Some("/secret/site-admin")
    );
}

/// The login page served by an admin built with `view_site_url` (or not).
async fn login_page(view_site_url: Option<&str>) -> String {
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .with_admin(|a| {
            let a = a.prefix("secret").path("/site-admin");
            match view_site_url {
                Some(url) => a.view_site_url(url),
                None => a,
            }
        })
        .build()
        .await
        .unwrap();
    let req = Request::builder()
        .uri("/secret/site-admin/login")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    String::from_utf8(to_bytes(resp.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap()
}

/// Below the sign-in button, a link back to the site: `view_site_url` when
/// set, else `/` — never the admin's own prefix.
#[tokio::test]
#[serial]
async fn the_login_page_links_back_to_the_site() {
    let page = login_page(Some("https://site.example")).await;
    assert!(
        page.contains(r#"<a class="login-back" href="https://site.example">"#),
        "{page}"
    );
    let page = login_page(None).await;
    assert!(page.contains(r#"<a class="login-back" href="/">"#), "{page}");
    assert!(!page.contains(r#"class="login-back" href="/secret"#));
    let label = runique::utils::trad::t("admin.login.back_to_site").into_owned();
    assert!(!label.is_empty() && label != "admin.login.back_to_site");
    assert!(page.contains(&label), "{page}");
}

/// Builds an admin whose list page is replaced by a template that doesn't
/// exist, with `dev` and `debug` as given.
async fn build_with_missing_page(dev: bool, debug: bool) -> Result<(), String> {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.dev = dev;
    config.debug = debug;
    // A valid production setup: only the missing page may refuse the boot.
    config.server.secret_key = "a-test-secret-key-of-at-least-32-chars".to_string();
    config.server.public_url = Some("https://example.test".to_string());
    RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .with_admin(|a| a.templates(|t| t.with_list("nowhere/list.html")))
        .build()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// A replaced admin page Tera never loaded refuses the boot in development or
/// debug, naming it; in production it is left to the first render.
#[tokio::test]
#[serial]
async fn a_missing_replaced_page_refuses_the_boot_in_dev_or_debug_only() {
    for (dev, debug) in [(true, false), (false, true), (true, true)] {
        let err = build_with_missing_page(dev, debug).await.unwrap_err();
        assert!(err.contains("nowhere/list.html"), "dev={dev} debug={debug}: {err}");
    }
    let prod = build_with_missing_page(false, false).await;
    assert!(prod.is_ok(), "production builds anyway: {prod:?}");
}
