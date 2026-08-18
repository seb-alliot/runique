// `.prefix()` et `.routes()` decrivent deux choses independantes :
//   .routes(admins::routes("/site-admin"))  -> ou vit l'admin
//   .prefix("secret")                       -> ce qui est monte devant
// L'URL publique est la composition des deux, quel que soit l'ordre d'appel.
// Avant, les deux ecrivaient dans `config.prefix` : le dernier appele gagnait,
// et l'ordre changeait silencieusement le comportement.

use axum::Router;
use axum::routing::get;
use runique::admin::AdminRoutes;
use runique::app::RuniqueApp;
use runique::auth::session::{AdminAuth, AdminLoginResult};
use runique::config::RuniqueConfig;
use sea_orm::{Database, DatabaseConnection};
use serial_test::serial;

// `#[serial]` sur tous les tests de ce fichier : chaque test construit une app
// admin complete, ce qui pousse ses routes nommees (admin_login, admin_dashboard...)
// dans le registre global `PENDING_URLS` avant de les drainer dans son propre
// engine via `add_urls()`. Ce registre est partage par tout le process de test —
// deux builds concurrents (y compris avec `test_robots_txt.rs`, egalement `#[serial]`)
// se volent mutuellement leurs entrees. Voir `register_url.rs`.

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

fn fake_admin_routes(path: &str) -> AdminRoutes {
    let p = path.trim_end_matches('/');
    let router = Router::new().route(&format!("{p}/ping"), get(|| async { "pong" }));
    AdminRoutes::new(p, router)
}

/// Construit l'app et rend l'engine, pour interroger le registre d'URLs nommees.
async fn build_app(mount: &str, admin_path: &str) -> runique::app::RuniqueApp {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;

    RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| {
            a.auth(MockAdminAuth)
                .prefix(mount)
                .routes(fake_admin_routes(admin_path))
        })
        .build()
        .await
        .unwrap()
}

/// `prefix_first` inverse l'ordre des deux appels dans la chaine du builder.
async fn build(mount: &str, admin_path: &str, prefix_first: bool) -> Router {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.disable())
        .with_admin(|a| {
            let a = a.auth(MockAdminAuth);
            if prefix_first {
                a.prefix(mount).routes(fake_admin_routes(admin_path))
            } else {
                a.routes(fake_admin_routes(admin_path)).prefix(mount)
            }
        })
        .build()
        .await
        .unwrap();

    app.router
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

/// Sans `.prefix()`, les URLs restent exactement celles de `routes()`.
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
