//! Builders de requêtes HTTP pour les tests oneshot.
//!
//! Deux niveaux :
//! - **HTTP oneshot** (`get`, `post`, …) : dispatch via `Router::oneshot` — tests intégration
//! - **Contexte handler** (`build_handler_req`) : construit le `runique::context::Request`
//!   directement — tests unitaires des handlers sans monter de serveur
//!
//! # Exemple oneshot
//! ```rust
//! use crate::helpers::{request, server::build_engine, server::build_default_router};
//!
//! #[tokio::test]
//! async fn mon_test() {
//!     let app = build_default_router(build_engine().await);
//!     let resp = request::get(app, "/").await;
//!     assert_eq!(resp.status(), 200);
//! }
//! ```
//!
//! # Exemple handler unitaire
//! ```rust
//! use crate::helpers::{db, request::build_handler_req, server::build_engine};
//!
//! #[tokio::test]
//! async fn mon_handler() {
//!     let engine = build_engine().await;
//!     let mut req = build_handler_req(engine, None, Default::default()).await;
//!     // appelle le handler directement avec &mut req
//! }
//! ```

use axum::{
    Router,
    body::Body,
    http::{Method, Request, header},
    response::Response,
    routing::get as axum_get,
};
use runique::{
    auth::session::CurrentUser,
    context::template::Request as HandlerReq,
    flash::Message,
    forms::Prisme,
    utils::{aliases::StrMap, middleware::csrf::CsrfToken},
};
use std::sync::{Arc, Mutex};
use tera::Context;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

// ── Handler context builder ───────────────────────────────────────────────────

/// Construit un `runique::context::Request` utilisable directement dans les tests de handlers.
///
/// La session est récupérée via un router oneshot minimal + `SessionManagerLayer` pour
/// garantir une vraie session tower-sessions (pas de constructeur public sur `Session`).
///
/// - `engine` : moteur de test (voir `server::build_engine()`)
/// - `user`   : utilisateur injecté (`None` = non authentifié)
/// - `body`   : données de formulaire simulées (vides par défaut)
///
/// CSRF marqué valide, méthode POST, session en mémoire isolée.
pub async fn build_handler_req(
    engine: Arc<runique::engine::RuniqueEngine>,
    user: Option<CurrentUser>,
    body: StrMap,
) -> HandlerReq {
    // Capture la session via un handler oneshot — seul moyen d'obtenir
    // une Session valide sans passer par le pipeline HTTP complet.
    let (tx, rx) = tokio::sync::oneshot::channel::<Session>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let app = Router::new()
        .route(
            "/",
            axum_get(move |session: Session| {
                let tx = tx.clone();
                async move {
                    if let Ok(mut g) = tx.lock()
                        && let Some(sender) = g.take()
                    {
                        let _ = sender.send(session);
                    }
                    "ok"
                }
            }),
        )
        .layer(SessionManagerLayer::new(MemoryStore::default()));

    let bootstrap = Request::builder()
        .method(Method::GET)
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let _ = app.oneshot(bootstrap).await;

    let session = rx.await.expect("session capture");

    let mut context = Context::new();
    context.insert("debug", &false);
    context.insert("csrf_token", "test-csrf-token");
    if let Some(ref u) = user {
        context.insert("current_user", u);
    }

    HandlerReq {
        engine,
        notices: Message {
            session: session.clone(),
        },
        session,
        csrf_token: CsrfToken("test-csrf-token".to_string()),
        context,
        method: Method::POST,
        path_params: Default::default(),
        query_params: Default::default(),
        user,
        prisme: Prisme {
            data: body,
            csrf_valid: true,
        },
    }
}

// ── GET ───────────────────────────────────────────────────────────────────────

/// Envoie une requête GET en oneshot sur `uri`.
pub async fn get(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête GET avec un header supplémentaire.
pub async fn get_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── POST ──────────────────────────────────────────────────────────────────────

/// Envoie une requête POST vide en oneshot sur `uri`.
pub async fn post(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête POST avec un header supplémentaire (ex: X-CSRF-Token).
pub async fn post_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── DELETE ────────────────────────────────────────────────────────────────────

/// Envoie une requête DELETE en oneshot sur `uri`.
pub async fn delete(app: Router, uri: &str) -> Response {
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

/// Envoie une requête DELETE avec un header supplémentaire.
pub async fn delete_with_header(app: Router, uri: &str, hdr: &str, value: &str) -> Response {
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header(hdr, value)
        .body(Body::empty())
        .unwrap();
    app.oneshot(req).await.unwrap()
}

// ── Bare request builders (pas de dispatch) ───────────────────────────────────

/// Construit une requête GET avec un header `Host` — utile pour les fonctions
/// qui inspectent les headers directement (ex: `is_localhost()`).
///
/// # Exemple
/// ```rust
/// use crate::helpers::request::build_with_host;
///
/// assert!(is_localhost(&build_with_host("localhost:3000")));
/// assert!(!is_localhost(&build_with_host("evil.com")));
/// ```
pub fn build_with_host(host: &str) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .header(header::HOST, host)
        .body(Body::empty())
        .unwrap()
}
