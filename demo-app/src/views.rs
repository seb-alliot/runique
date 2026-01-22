use crate::forms::{Blog as BlogForm, UsernameForm};
use crate::models::model_derive;
use crate::models::users;
use crate::models::users::Entity as UserEntity;
use runique::prelude::*;
use runique::{error, flash_now, info, success, warning};
use runique::serde_json;



/// Page d'accueil
pub async fn index(mut template: TemplateContext) -> Response {
    template.context_update(context_update!{
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "Axum pour le backend",
        "template" => "Tera comme moteur de templates",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions",
    }).render("index.html")
}

/// Formulaire d'inscription
pub async fn inscription(mut template: TemplateContext) -> Response {
    let form = model_derive::ModelForm::build(template.engine.tera.clone());

    template.context_update(context_update!{
        "title" => "Profil utilisateur",
        "inscription_form" => form,
    }).render("inscription_form.html")
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut ctx: RuniqueContext,
    mut template: TemplateContext,
    ExtractForm(mut form): ExtractForm<model_derive::ModelForm>,
) -> Response {
    let db = ctx.engine.db.clone();

    if form.is_valid().await {
        match form.save(&*db).await {
            Ok(user) => {
                success!(ctx.flash => format!("Bienvenue {}, votre compte a été créé !", user.username));
                return Redirect::to("/").into_response();
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                return template.context_update(context_update!{
                    "title" => "Erreur de base de données",
                    "inscription_form" => form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous"),
                }).render("inscription_form.html");
            }
        }
    }

    template.context_update(context_update!{
        "title" => "Erreur de validation",
        "inscription_form" => form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    }).render("inscription_form.html")
}

/// Formulaire de recherche d'utilisateur
pub async fn search_user_form(mut template: TemplateContext) -> Response {
    let form = UsernameForm::build(template.engine.tera.clone());
    template.context_update(context_update!{
        "title" => "Rechercher un utilisateur",
        "user" => form,
    }).render("profile/view_user.html")
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
        Ok(Some(user)) => {
            template.context_update(context_update!{
                "title" => "Vue utilisateur",
                "username" => user.username,
                "email" => user.email,
                "user" => user,
                "messages" => flash_now!(warning => &user.username),
            }).render("profile/view_user.html")
        }
        Ok(None) => {
            template.context_update(context_update!{
                "title" => "Utilisateur non trouvé",
                "user" => form,
                "messages" => flash_now!(error => format!("Utilisateur {} non trouvé", username)),
            }).render("profile/view_user.html")
        }
        Err(_) => template.render_500("Erreur lors de la recherche"),
    }
}

/// Blog form
pub async fn blog_form(mut template: TemplateContext) -> Response {
    let form = BlogForm::build(template.engine.tera.clone());
    template.context_update(context_update!{
        "title" => "Formulaire Blog",
        "blog_form" => form,
    }).render("blog/blog.html")
}

/// Page "À propos"
pub async fn about(mut template: TemplateContext) -> Response {
    success!(template.flash => "Ceci est un message de succès.");
    info!(template.flash => "Ceci est un message d'information.");

    template.context_update(context_update!{
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    }).render("about/about.html")
}



/// Teste Csrf

pub async fn test_csrf(mut template: TemplateContext) -> Response {
    success!(template.flash => "CSRF token validé avec succès !");
    Redirect::to("/").into_response()
}