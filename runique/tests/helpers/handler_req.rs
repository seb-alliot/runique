//! `build_handler_req` : construit le `runique::context::Request` directement —
//! tests unitaires des handlers sans monter de serveur. Passe par
//! `Prisme::for_test`, d'où la feature `test-utils`.
//!
//! # Exemple
//! ```rust
//! use crate::helpers::{handler_req::build_handler_req, server::build_engine};
//! use axum::http::Method;
//!
//! #[tokio::test]
//! async fn mon_handler() {
//!     let engine = build_engine().await;
//!     let mut req = build_handler_req(engine, None, Default::default(), Method::POST).await;
//!     // appelle le handler directement avec &mut req
//! }
//! ```

use axum::{
    Router,
    body::Body,
    http::{Method, Request},
    routing::get as axum_get,
};
use runique::{
    auth::session::CurrentUser,
    context::template::Request as HandlerReq,
    flash::Message,
    forms::Prisme,
    utils::{aliases::StrMap, crypto::csrf::CsrfToken},
};
use std::sync::{Arc, Mutex};
use tera::Context;
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

/// Construit un `runique::context::Request` utilisable directement dans les tests de handlers.
///
/// La session est récupérée via un router oneshot minimal + `SessionManagerLayer` pour
/// garantir une vraie session tower-sessions (pas de constructeur public sur `Session`).
///
/// - `engine` : moteur de test (voir `server::build_engine()`)
/// - `user`   : utilisateur injecté (`None` = non authentifié)
/// - `body`   : données de formulaire simulées (vides par défaut)
/// - `method` : méthode HTTP simulée
///
/// CSRF marqué valide, session en mémoire isolée.
pub async fn build_handler_req(
    engine: Arc<runique::engine::RuniqueEngine>,
    user: Option<CurrentUser>,
    body: StrMap,
    method: Method,
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
        method,
        path_params: Default::default(),
        raw_query: String::new(),
        query_params: Default::default(),
        user,
        prisme: Prisme::for_test(body, true),
        headers: Default::default(),
        honeypot_field_name: None,
    }
}
