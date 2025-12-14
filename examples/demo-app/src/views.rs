use rusti::{
    Context, Response, processor::message_processor::{Message, Template}, reverse
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
    mut message: Message,

) -> Response {
    message.success( "Ceci est un message de succès de test.").await.unwrap();
    message.info("Ceci est un message d'information de test.").await.unwrap();
    message.error("Ceci est un message d'erreur de test.").await.unwrap();

    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
        "index": reverse("index").unwrap_or_default(),
    })).unwrap_or_default();

    template.render("about/about.html", &context)
}