use rusti::{
    ColumnTrait, DatabaseConnection, ExtractForm, IntoResponse, Message, Redirect, Response, RustiForm, Template, context, reverse_with_parameters
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

pub async fn user_profile(
    template: Template,
    ExtractForm(form): ExtractForm<ModelForm>,
) -> Response {
    let ctx = context!{
        "title", "Profil Utilisateur";
        "form", form
    };
    template.render("profile/register_profile.html", &ctx)
}

/// GET - Afficher le profil utilisateur
pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    if user.is_valid() {
        match user.save(&*db).await {
            Ok(created_user) => {
                message.success("Profil utilisateur créé avec succès !").await.unwrap();
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
                    "form", ModelForm::build();
                    "forms_errors", user.get_errors();
                    "title", "Profil";
                    "db_error", error_msg
                };
                return template.render("profile/register_profile.html", &ctx);
            }
        }
    }
    message.error("Veuillez corriger les erreurs du formulaire").await.unwrap();
    let ctx = context!{
        "form", ModelForm::build();  
        "forms_errors", user.get_errors();  
        "title", "Erreur de validation"
    };
    template.render("profile/register_profile.html", &ctx)
}

/// GET - Affiche la page avec le formulaire de recherche
pub async fn user(template: Template) -> Response {
    let ctx = context!{
        "title", "Rechercher un utilisateur";
        "form", UsernameForm::build()
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
                "age", user.age;
                "form", UsernameForm::build()
            };
            template.render("profile/view_user.html", &ctx)
        }
        Ok(None) => {
            // Aucun utilisateur trouvé - 200 OK avec message
            let ctx = context!{
                "title", "Utilisateur non trouvé";
            };
            template.render("profile/view_user.html", &ctx)
        }
        Err(_) => {
            // Erreur base de données - 500
            template.render_500("Erreur lors de la recherche")
        }
    }
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


/// Ajax test CSRF
pub async fn test_csrf(mut message: Message) -> Response {
    message.success("Requête POST avec CSRF réussie !").await.unwrap();
    Redirect::to("/").into_response()
}