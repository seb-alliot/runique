// Tests pour flash_manager (Message extractor + push/get_all)
//
// Stratégie : serveur persistant (OnceLock) avec session MemoryStore.
// Chaque test crée son propre client reqwest (cookie jar isolé → session distincte).

use axum::{Json, Router, routing::get, routing::post};
use runique::flash::{FlashMessage, flash_manager::Message};
use std::{net::SocketAddr, sync::OnceLock};
use tower_sessions::{MemoryStore, SessionManagerLayer};

// ═══════════════════════════════════════════════════════════════
// Serveur de test partagé
// ═══════════════════════════════════════════════════════════════

static FLASH_SERVER: OnceLock<SocketAddr> = OnceLock::new();

fn flash_server_addr() -> SocketAddr {
    *FLASH_SERVER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime flash test");
            rt.block_on(async {
                let session_layer = SessionManagerLayer::new(MemoryStore::default());

                let app = Router::new()
                    .route(
                        "/push/success",
                        post(|msg: Message| async move {
                            msg.success("Test succès").await;
                            "ok"
                        }),
                    )
                    .route(
                        "/push/error",
                        post(|msg: Message| async move {
                            msg.error("Test erreur").await;
                            "ok"
                        }),
                    )
                    .route(
                        "/push/info",
                        post(|msg: Message| async move {
                            msg.info("Test info").await;
                            "ok"
                        }),
                    )
                    .route(
                        "/push/warning",
                        post(|msg: Message| async move {
                            msg.warning("Test avertissement").await;
                            "ok"
                        }),
                    )
                    .route(
                        "/get",
                        get(|msg: Message| async move { Json(msg.get_all().await) }),
                    )
                    .layer(session_layer);

                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind flash server");
                let addr = listener.local_addr().expect("local addr");
                tx.send(addr).expect("send addr");
                axum::serve(listener, app).await.expect("serve");
            });
        });

        rx.recv().expect("recv addr")
    })
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("reqwest client")
}

// ═══════════════════════════════════════════════════════════════
// Tests — push et get_all
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_flash_success_push_et_get() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/success"))
        .send()
        .await
        .unwrap();

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test succès");
    assert!(matches!(
        messages[0].level,
        runique::flash::MessageLevel::Success
    ));
}

#[tokio::test]
async fn test_flash_error_push_et_get() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/error"))
        .send()
        .await
        .unwrap();

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test erreur");
    assert!(matches!(
        messages[0].level,
        runique::flash::MessageLevel::Error
    ));
}

#[tokio::test]
async fn test_flash_info_push_et_get() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/info"))
        .send()
        .await
        .unwrap();

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test info");
    assert!(matches!(
        messages[0].level,
        runique::flash::MessageLevel::Info
    ));
}

#[tokio::test]
async fn test_flash_warning_push_et_get() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/warning"))
        .send()
        .await
        .unwrap();

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Test avertissement");
    assert!(matches!(
        messages[0].level,
        runique::flash::MessageLevel::Warning
    ));
}

// ═══════════════════════════════════════════════════════════════
// Tests — comportement flash (lecture unique)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_flash_get_all_vide_sans_push() {
    let addr = flash_server_addr();
    let c = client(); // nouvelle session, aucun push

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        messages.is_empty(),
        "get_all sans push préalable doit retourner une liste vide"
    );
}

#[tokio::test]
async fn test_flash_get_all_efface_apres_lecture() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/info"))
        .send()
        .await
        .unwrap();

    // Première lecture : obtient le message
    let first: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first.len(), 1, "Première lecture doit retourner 1 message");

    // Deuxième lecture : doit être vide (effet flash)
    let second: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        second.is_empty(),
        "Les messages doivent être effacés après get_all()"
    );
}

#[tokio::test]
async fn test_flash_accumulation_multiple_push() {
    let addr = flash_server_addr();
    let c = client();

    c.post(format!("http://{addr}/push/success"))
        .send()
        .await
        .unwrap();
    c.post(format!("http://{addr}/push/error"))
        .send()
        .await
        .unwrap();
    c.post(format!("http://{addr}/push/info"))
        .send()
        .await
        .unwrap();

    let messages: Vec<FlashMessage> = c
        .get(format!("http://{addr}/get"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        messages.len(),
        3,
        "Trois push successifs doivent accumuler 3 messages"
    );
}

// ═══════════════════════════════════════════════════════════════
// Test — extraction sans session (rejection)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_flash_message_extractor_sans_session_retourne_500() {
    // Router SANS SessionManagerLayer → Message extractor ne trouve pas de Session
    // → from_request_parts retourne Err(StatusCode::INTERNAL_SERVER_ERROR)
    let app = Router::new().route(
        "/get",
        get(|msg: Message| async move { Json(msg.get_all().await) }),
    );
    // Pas de .layer(session_layer)

    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let req = Request::builder()
        .method("GET")
        .uri("/get")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        500,
        "Sans session, l'extracteur doit rejeter avec 500"
    );
}
