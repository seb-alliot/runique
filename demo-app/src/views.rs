use crate::forms::{Blog as BlogForm, UsernameForm};
use crate::models::model_derive;
use crate::models::users;
use crate::models::users::Entity as UserEntity;
use runique::prelude::*;
use runique::serde_json;
use runique::{error, flash_now, info, success, warning};

/// Page d'accueil
pub async fn index(mut template: TemplateContext) -> Result<Response, AppError> {
    template
        .context(context_update! {
            "title" => "Bienvenue sur Runique",
            "description" => "Un framework web moderne inspiré de Django",
            "status" => "Framework en cours de développement...",
            "backend" => "Axum pour le backend",
            "template" => "Tera comme moteur de templates",
            "tokio" => "Runtime asynchrone tokio",
            "session" => "Tower pour la gestion des sessions",
        })
        .render("index.html")
}

/// Formulaire d'inscription
pub async fn inscription(mut template: TemplateContext) -> Result<Response, AppError> {
    let form = model_derive::ModelForm::build(template.engine.tera.clone());

    template
        .context(context_update! {
            "title" => "Profil utilisateur",
            "inscription_form" => form,
        })
        .render("inscription_form.html")
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut template: TemplateContext,
    ExtractForm(mut form): ExtractForm<model_derive::ModelForm>,
) -> Response {
    let db = template.engine.db.clone();
    if form.is_valid().await {
        match form.save(&*db).await {
            Ok(user) => {
                success!(template.messages => format!("Bienvenue {}, votre compte a été créé !", user.username));
                return Redirect::to("/").into_response();
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                return template.context(context_update!{
                    "title" => "Erreur de base de données",
                    "inscription_form" => form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous"),
                }).render("inscription_form.html").into_response();
            }
        }
    }

    template
        .context(context_update! {
            "title" => "Erreur de validation",
            "inscription_form" => form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
        })
        .render("inscription_form.html")
        .into_response()
}

/// Formulaire de recherche d'utilisateur
pub async fn search_user_form(mut template: TemplateContext) -> Result<Response, AppError> {
    let form = UsernameForm::build(template.engine.tera.clone());

    template
        .context(context_update! {
            "title" => "Rechercher un utilisateur",
            "user" => form,
        })
        .render("profile/view_user.html")
}

/// Exemple pour chercher un utilisateur
pub async fn info_user(
    mut template: TemplateContext,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Response {
    let username = form.get_form().get_value("username").unwrap_or_default();
    let db = template.engine.db.clone();

    match UserEntity::objects
        .filter(users::Column::Username.eq(&username))
        .first(&db)
        .await
    {
        Ok(Some(user)) => template
            .context(context_update! {
                "title" => "Vue utilisateur",
                "username" => user.username,
                "email" => user.email,
                "user" => user,
                "messages" => flash_now!(warning => &user.username),
            })
            .render("profile/view_user.html")
            .into_response(),
        Ok(None) => template
            .context(context_update! {
                "title" => "Utilisateur non trouvé",
                "messages" => flash_now!(error => "L'utilisateur n'a pas été trouvé"),
            })
            .render("profile/view_user.html")
            .into_response(),
        Err(_err) => template
            .context(context_update! {
                "title" => "Erreur de base de données",
                "messages" => flash_now!(error => "Une erreur est survenue lors de la recherche"),
            })
            .render("profile/view_user.html")
            .into_response(),
    }
}

/// Blog form
pub async fn blog_form(mut template: TemplateContext) -> Result<Response, AppError> {
    let form = BlogForm::build(template.engine.tera.clone());
    template
        .context(context_update! {
            "title" => "Formulaire Blog",
            "blog_form" => form,
        })
        .render("blog/blog.html")
}

/// Page "À propos"
pub async fn about(mut template: TemplateContext) -> Response {
    success!(template.messages => "Ceci est un message de succès." );
    info!(template.messages => "Ceci est un message d'information." );

    template.context(context_update!{
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    }).render("about/about.html").into_response()
}

/// Teste Csrf

pub async fn test_csrf(mut template: TemplateContext) -> Response {
    success!(template.messages => "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}
