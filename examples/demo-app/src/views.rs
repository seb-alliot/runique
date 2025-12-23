use rusti::{
    Context,
    FormulaireAxumForm,
    IntoResponse,
    Message,
    Path,
    Redirect,
    Response,
    Template,
    json,
    reverse_with_parameters,
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
    let context: Context = Context::from_serialize(json!({
        "title": format!("Bienvenue {}, ton Id est {}", name, id),
    })).unwrap_or_default();

    template.render("profile/profile.html", &context)
}

/// POST - Traiter le formulaire de profil
pub async fn user_profile_submit(
    Path((id, name)): Path<(u32, String)>,
    mut message: Message,
    template: Template,
    FormulaireAxumForm(user): FormulaireAxumForm<UserForm>,
) -> Response {

    if user.is_valid() {
        let username: Option<String> = user.internal.get_value("username");
        let email: Option<String> = user.internal.get_value("email");
        let age: Option<i32> = user
            .get_value::<i64>("age")
            .map(|v| v as i32);
        let password: Option<String> = user.internal.get_value("password");
        let info = format!("ID: {}, Name: {}", id, name);
        let content = format!("Ton pseudo {}, email {}, age {}, et mot de passe {}  ont étais valider", username.unwrap_or_default(), email.unwrap_or_default(), age.unwrap_or_default(), password.unwrap_or_default());
        message.success(&content).await.unwrap();
        message.info(&info).await.unwrap();
        let target = reverse_with_parameters("user_profile", &[("id", &id.to_string()), ("name", &name)]).unwrap();
                Redirect::to(&target).into_response() // Pas de return + ; ici, c'est l'expression de sortie
            }
            else {
                // On regroupe le is_not_valid et le else ici
                if user.is_not_valid() {
                    message.error("Veuillez corriger les erreurs.").await.unwrap();
                }

                let context = Context::from_serialize(json!({
                    "form": &user,
                    "title": &format!("Erreur sur le profil de {}", name),
                })).unwrap_or_default();

                template.render("profile/profile.html", &context)
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