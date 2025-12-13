use rusti::{
    Context,
    middleware::flash_message::{flash_success, flash_error, flash_info},
    Response,
    Session,
    processor::message_processor::Template,
};
use serde_json::json;

/// Page d'accueil
pub async fn index(
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",
        "description": "Un framework web moderne inspiré de Django",
    })).unwrap_or_default();

    template.render("base.html", &context)
}

/// Page "À propos"
pub async fn about(
    template: Template,
    mut session: Session,
) -> Response {
    let _ = flash_success(&mut session, "Ceci est un message de succès de test.").await;
    let _ = flash_info(&mut session, "Ceci est un message d'information de test.").await;
    let _ = flash_error(&mut session, "Ceci est un message d'erreur de test.").await;

    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
    })).unwrap_or_default();

    template.render("about/about.html", &context)
}