// Tests pour load_user_middleware
//
// Stratégie : serveur persistant (OnceLock) avec MemoryStore.
// Chaque test crée son propre client reqwest (cookie jar isolé → session distincte).

use axum::{Extension, Router, middleware, routing::get, routing::post};
use runique::auth::{CurrentUser, load_user_middleware, login};
use sea_orm::DatabaseConnection;
use std::{net::SocketAddr, sync::Arc, sync::OnceLock};
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
                let db = sea_orm::Database::connect("sqlite::memory:")
                    .await
                    .expect("sqlite:memory");
                let db: Arc<DatabaseConnection> = Arc::new(db);

                let store = MemoryStore::default();
                let session_layer = SessionManagerLayer::new(store).with_secure(false);

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
                    .layer(middleware::from_fn_with_state(db.clone(), load_user_middleware));

                // Routes publiques (pas de middleware auth)
                let public = Router::new()
                    .route(
                        "/do_login_full",
                        post(
                            |session: Session,
                            Extension(db): Extension<Arc<DatabaseConnection>>| async move {
                                login(&session, &db, 2, "bob", true, false, None, false).await.unwrap();
                                "ok"
                            },
                        ),
                    );

                let app = Router::new()
                    .merge(user_area)
                    .merge(public)
                    .layer(Extension(db))
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
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
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

    c.post(format!("http://{addr}/do_login_full"))
        .send()
        .await
        .unwrap();

    let resp = c.get(format!("http://{addr}/whoami")).send().await.unwrap();

    assert_eq!(resp.text().await.unwrap(), "bob");
}
