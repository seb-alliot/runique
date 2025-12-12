use rusti::{
    Context, Extension, FlashMessageSession, Message, Response, Session, Settings, StatusCode, Tera
};

use rusti::middleware::error_handler::render_template;
use std::sync::Arc;
use serde_json::json;

/// Page d'accueil
pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
    mut session: Session,
) -> Response {

    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",
        "description": "Un framework web moderne inspiré de Django",
        "debug": config.debug,
        // Récupérer et consommer les messages flash de la session
        "messages": session.remove_messages().await.ok(),
    })).unwrap_or_default();

    render_template(&tera, "base.html", &context, StatusCode::OK, &config)
}

/// Page "À propos"
pub async fn about(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
    mut session: Session,
) -> Response {

    // Ajouter des messages flash de test
    session.insert_message(Message::success("Ceci est un message de succès de test.")).await.unwrap();
    session.insert_message(Message::error("Ceci est un message d'erreur de test.")).await.unwrap();
    session.insert_message(Message::info("Ceci est un message d'information de test.")).await.unwrap();

    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
        "debug": config.debug,
    })).unwrap_or_default();

    render_template(&tera, "about/about.html", &context, StatusCode::OK, &config)
}
