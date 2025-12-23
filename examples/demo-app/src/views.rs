use rusti::{
    Context,
    Message,
    Path,
    Response,
    Template,
    json,
    AxumForm,
    Redirect,
    IntoResponse,
};
use crate::form::UserForm;


/// Page d'accueil
pub async fn index(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",
        "description": "Un framework web moderne inspiré de Django",
        "status": "Framework en cours de développement...",
        "backend": "J'utilise axum pour le backend",
        "template": "Tera pour moteur de templates.",
        "tokio": "Le runtime asynchrone tokio",
        "session": "Tower pour la gestion des sessions.",
    })).unwrap_or_default();

    template.render("index.html", &context)
}

/// Page "À propos"
pub async fn about(
    template: Template,
    mut message: Message,
) -> Response {
    message.success("Ceci est un message de succès de test.").await.unwrap();
    message.info("Ceci est un message d'information de test.").await.unwrap();
    message.error("Ceci est un message d'erreur de test.").await.unwrap();

    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
    })).unwrap_or_default();

    template.render("about/about.html", &context)
}

/// GET - Afficher le profil utilisateur
pub async fn user_profile(
    Path((id, name)): Path<(u32, String)>,
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": format!("Bienvenue {}, ton Id est {}", name, id),
        "user_id": id,
        "name": name,
    })).unwrap_or_default();

    template.render("profile/profile.html", &context)
}

/// POST - Traiter le formulaire de profil
pub async fn user_profile_submit(
    Path((id, name)): Path<(u32, String)>,
    mut message: Message,
    // On ajoute l'extracteur de données brutes d'Axum
    AxumForm(raw): AxumForm<UserForm>,
) -> Response {

    if raw.is_valid() {
        message.success("Profil mis à jour avec succès !").await.unwrap();
        Redirect::to(&format!("/user/{}/{}", id, name)).into_response()
    } else if raw.is_not_valid() {
        message.error("Erreur lors de la mise à jour du profil.").await.unwrap();
        Redirect::to(&format!("/user/{}/{}", id, name)).into_response()
    } else {
        message.info("Aucune modification détectée dans le profil.").await.unwrap();
        Redirect::to(&format!("/user/{}/{}", id, name)).into_response()
    }

}

/// Page spéciale sapin de Noël
pub async fn about_sapin(
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Sapin de Noël avec Rusti",
    })).unwrap_or_default();

    template.render("sapin/sapin.html", &context)
}

/// Page de test CSRF
pub async fn test_csrf(
    mut message: Message,
) -> Response {
    message.success("Requête POST avec CSRF réussie !").await.unwrap();
    Redirect::to("/").into_response()
}