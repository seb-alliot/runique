use rusti::{
    ExtractForm,
    IntoResponse,
    Message,
    Redirect,
    Response,
    Template,
    reverse_with_parameters,
    DatabaseConnection,
    context,
    ColumnTrait,
};
//     get_or_return, a utilisé pour récupérer ou retourner une réponse d'erreur

use rusti::axum::Extension;
use std::sync::Arc;
use crate::models::users::ModelForm;
use crate::models::users;
use crate::models::users::Entity as User;
use crate::forms::UsernameForm;

/// Page d'accueil
pub async fn index(template: Template) -> Response {
    let ctx = context!{
        "title", "Bienvenue sur Rusti";
        "description", "Un framework web moderne inspiré de Django";
        "status", "Framework en cours de développement...";
        "backend", "J'utilise axum pour le backend";
        "template", "Tera pour moteur de templates.";
        "tokio", "Le runtime asynchrone tokio";
        "session", "Tower pour la gestion des sessions."};

    template.render("index.html", &ctx)
}

/// Page "À propos"
pub async fn about(
    template: Template,
    mut message: Message,
) -> Response {
    message.success("Ceci est un message de succès de test.").await.unwrap();
    message.info("Ceci est un message d'information de test.").await.unwrap();
    message.error("Ceci est un message d'erreur de test.").await.unwrap();

    let ctx = context!{
        "title", "À propos de Rusti Framework";
        "content", "Rusti est un framework web inspiré de Django, construit sur Axum et Tera."
    };
    template.render("about/about.html", &ctx)
}

/// GET - Afficher le profil utilisateur
pub async fn user_profile(
    template: Template,
) -> Response {
    let ctx = context!()
        .add("title", "Profil Utilisateur");

    template.render("profile/register_profile.html", &ctx)
}

/// POST - Traiter le formulaire de profil
pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    if user.is_valid() {
        match user.save(&*db).await {
            Ok(created_user) => {
                message.success(format!(
                    "Utilisateur {} créé avec ID {} !",
                    created_user.username,
                    created_user.id
                )).await.unwrap();

                let target = reverse_with_parameters(
                    "user_profile",
                    &[("id", &created_user.id.to_string()), ("name", &created_user.username)]
                ).unwrap();

                return Redirect::to(&target).into_response();
            }
            Err(err) => {
                let error_msg = if err.to_string().contains("unique") {
                    if err.to_string().contains("username") {
                        "Ce nom d'utilisateur existe déjà !"
                    } else if err.to_string().contains("email") {
                        "Cet email est déjà utilisé !"
                    } else {
                        "Cette valeur existe déjà dans la base de données"
                    }
                } else {
                    "Erreur lors de la sauvegarde"
                };

                message.error(error_msg).await.unwrap();

                let ctx = context!{
                "form", &user;
                "title", "Profil";
                "db_error", error_msg
                };
                return template.render("profile/register_profile.html", &ctx);
            }
        }
    }

    message.error("Veuillez corriger les erreurs du formulaire").await.unwrap();

    let ctx = context!{
        "form", &user;
        "title", "Erreur de validation"
    };

    template.render("profile/register_profile.html", &ctx)
}

/// GET - Affiche la page avec le formulaire de recherche
pub async fn user(template: Template) -> Response {
    let ctx = context!{
        "title", "Rechercher un utilisateur"
    };
    template.render("profile/view_user.html", &ctx)
}

/// POST - Traite le formulaire de recherche et affiche les résultats
pub async fn view_user(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Response {
    let name: String = form.form.get_value("username").unwrap_or_default();

    // Rechercher l'utilisateur dans la base de données
    match User::objects
        .filter(users::Column::Username.eq(&name))
        .first(&db)
        .await
    {
        Ok(Some(user)) => {
            // Utilisateur trouvé
            let ctx = context!{
                "title", "Vue Utilisateur";
                "username", &user.username;
                "email", &user.email;
                "age", user.age
            };
            template.render("profile/view_user.html", &ctx)
        }
        Ok(None) => {
            // Aucun utilisateur trouvé - 200 OK avec message
            let ctx = context!{
                "title", "Utilisateur non trouvé";
                "searched_username", &name;
                "message", format!("Aucun utilisateur trouvé pour '{}'", name)
            };
            template.render("profile/view_user.html", &ctx)
        }
        Err(_) => {
            // Erreur base de données - 500
            template.render_500("Erreur lors de la recherche")
        }
    }
}

/// Page spéciale sapin de Noël
pub async fn about_sapin(template: Template) -> Response {
    let ctx = context!()
    .add("title", "Sapin de Noël avec Rusti");

    template.render("sapin/sapin.html", &ctx)
}

/// Ajax test CSRF
pub async fn test_csrf(mut message: Message) -> Response {
    message.success("Requête POST avec CSRF réussie !").await.unwrap();
    Redirect::to("/").into_response()
}