// Tests pour login_required, redirect_if_authenticated, load_user_middleware, has_permission
//
// Stratégie : serveur persistant (OnceLock) avec MemoryStore.
// Chaque test crée son propre client reqwest (cookie jar isolé → session distincte).

use axum::{middleware, routing::get, routing::post, Extension, Router};
use runique::middleware::auth::{
    has_permission, load_user_middleware, login, login_required, login_staff,
    redirect_if_authenticated, CurrentUser,
};
use std::{net::SocketAddr, sync::OnceLock};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

// ═══════════════════════════════════════════════════════════════
// Serveur de test partagé
// ═══════════════════════════════════════════════════════════════

static AUTH_MW_SERVER: OnceLock<SocketAddr> = OnceLock::new();

fn auth_mw_addr() -> SocketAddr {
    *AUTH_MW_SERVER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("rt auth_mw");
            rt.block_on(async {
                let store = MemoryStore::default();
                let session_layer = SessionManagerLayer::new(store).with_secure(false);

                // Routes protégées par login_required
                let protected = Router::new()
                    .route("/protected", get(|| async { "secret" }))
                    .layer(middleware::from_fn(login_required));

                // Routes avec redirect_if_authenticated
                let login_page = Router::new()
                    .route("/login_page", get(|| async { "login page" }))
                    .layer(middleware::from_fn(redirect_if_authenticated));

                // Routes avec load_user_middleware
                let user_area = Router::new()
                    .route(
                        "/whoami",
                        get(|ext: Option<Extension<CurrentUser>>| async move {
                            match ext {
                                Some(Extension(u)) => u.username,
                                None => "anonymous".to_string(),
                            }
                        }),
                    )
                    .layer(middleware::from_fn(load_user_middleware));

                // Routes publiques (pas de middleware auth)
                let public = Router::new()
                    .route(
                        "/do_login",
                        post(|session: Session| async move {
                            login(&session, 1, "alice").await.unwrap();
                            "ok"
                        }),
                    )
                    .route(
                        "/do_login_full",
                        post(|session: Session| async move {
                            login_staff(
                                &session,
                                2,
                                "bob",
                                true,
                                false,
                                vec!["editor".to_string()],
                            )
                            .await
                            .unwrap();
                            "ok"
                        }),
                    )
                    .route(
                        "/has_perm",
                        get(|session: Session| async move {
                            if has_permission(&session, "any").await {
                                "yes"
                            } else {
                                "no"
                            }
                        }),
                    );

                let app = Router::new()
                    .merge(protected)
                    .merge(login_page)
                    .merge(user_area)
                    .merge(public)
                    .layer(session_layer);

                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind auth_mw");
                let addr = listener.local_addr().unwrap();
                tx.send(addr).unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });

        rx.recv().unwrap()
    })
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none()) // ne pas suivre les redirects
        .build()
        .unwrap()
}

// ═══════════════════════════════════════════════════════════════
// login_required
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_login_required_anonymous_retourne_redirect() {
    let addr = auth_mw_addr();
    let c = client();

    let resp = c
        .get(format!("http://{addr}/protected"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "anonyme doit être redirigé (got {})",
        resp.status()
    );
}

#[tokio::test]
async fn test_login_required_authentifie_passe() {
    let addr = auth_mw_addr();
    let c = client();

    // Login
    c.post(format!("http://{addr}/do_login"))
        .send()
        .await
        .unwrap();

    // Accès protégé
    let resp = c
        .get(format!("http://{addr}/protected"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "secret");
}

// ═══════════════════════════════════════════════════════════════
// redirect_if_authenticated
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_redirect_if_auth_anonyme_passe() {
    let addr = auth_mw_addr();
    let c = client();

    let resp = c
        .get(format!("http://{addr}/login_page"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "login page");
}

#[tokio::test]
async fn test_redirect_if_auth_connecte_redirige() {
    let addr = auth_mw_addr();
    let c = client();

    // Login
    c.post(format!("http://{addr}/do_login"))
        .send()
        .await
        .unwrap();

    // Page de login → doit rediriger
    let resp = c
        .get(format!("http://{addr}/login_page"))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_redirection(),
        "utilisateur connecté doit être redirigé (got {})",
        resp.status()
    );
}

// ═══════════════════════════════════════════════════════════════
// load_user_middleware
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_load_user_anonyme_pas_dextension() {
    let addr = auth_mw_addr();
    let c = client();

    let resp = c.get(format!("http://{addr}/whoami")).send().await.unwrap();

    assert_eq!(resp.text().await.unwrap(), "anonymous");
}

#[tokio::test]
async fn test_load_user_connecte_injecte_current_user() {
    let addr = auth_mw_addr();
    let c = client();

    // Login complet avec bob
    c.post(format!("http://{addr}/do_login_full"))
        .send()
        .await
        .unwrap();

    let resp = c.get(format!("http://{addr}/whoami")).send().await.unwrap();

    assert_eq!(resp.text().await.unwrap(), "bob");
}

// ═══════════════════════════════════════════════════════════════
// has_permission
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_anonyme_false() {
    let addr = auth_mw_addr();
    let c = client();

    let resp = c
        .get(format!("http://{addr}/has_perm"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.text().await.unwrap(), "no");
}

#[tokio::test]
async fn test_has_permission_connecte_true() {
    let addr = auth_mw_addr();
    let c = client();

    c.post(format!("http://{addr}/do_login"))
        .send()
        .await
        .unwrap();

    let resp = c
        .get(format!("http://{addr}/has_perm"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.text().await.unwrap(), "yes");
}
