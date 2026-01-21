use crate::forms::{Blog as BlogForm, UsernameForm};
use crate::models::model_derive;
use crate::models::users;
use crate::models::users::Entity as UserEntity;
use runique::prelude::*;
use runique::{context, error, flash_now, info, success, warning}; // Macros explicites

/// Page d'accueil
pub async fn index(_ctx: RuniqueContext, template: TemplateContext) -> Response {
    let ctx_tmpl = context! {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "Axum pour le backend",
        "template" => "Tera comme moteur de templates",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions"
    };
    template.render("index.html", &ctx_tmpl)
}

/// Formulaire d'inscription
pub async fn inscription(ctx: RuniqueContext, template: TemplateContext) -> Response {
    let form = model_derive::ModelForm::build(ctx.engine.tera.clone());
    let ctx_tmpl = context! {
        "title" => "Profil utilisateur",
        "inscription_form" => form
    };
    template.render("inscription_form.html", &ctx_tmpl)
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut ctx: RuniqueContext,
    template: TemplateContext,
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
                let ctx_tmpl = context! {
                    "title" => "Erreur de base de données",
                    "inscription_form" => form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                return template.render("inscription_form.html", &ctx_tmpl);
            }
        }
    }

    let ctx_tmpl = context! {
        "title" => "Erreur de validation",
        "inscription_form" => form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous")
    };
    template.render("inscription_form.html", &ctx_tmpl)
}

/// Formulaire de recherche d'utilisateur
pub async fn search_user_form(ctx: RuniqueContext, template: TemplateContext) -> Response {
    let form = UsernameForm::build(ctx.engine.tera.clone());
    let ctx_tmpl = context! {
        "title" => "Rechercher un utilisateur",
        "user" => form
    };
    template.render("profile/view_user.html", &ctx_tmpl)
}

/// Exemple pour chercher un utilisateur
pub async fn info_user(
    ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(form): ExtractForm<UsernameForm>,
) -> Response {
    let username = form.get_form().get_value("username").unwrap_or_default();

    let db = ctx.engine.db.clone();

    match UserEntity::objects
        .filter(users::Column::Username.eq(&username))
        .first(&db)
        .await
    {
        Ok(Some(user)) => {
            let messages = flash_now!(warning => &user.username);
            let mut ctx_tmpl = context! {
                "title" => "Vue utilisateur",
                "username" => &user.username,
                "email" => &user.email,
                "user" => &user
            };
            ctx_tmpl.insert("messages", &messages);
            template.render("profile/view_user.html", &ctx_tmpl)
        }
        Ok(None) => {
            let messages = flash_now!(error => format!("Utilisateur {} non trouvé", username));
            let mut ctx_tmpl = context! {
                "title" => "Utilisateur non trouvé",
                "user" => &form
            };
            ctx_tmpl.insert("messages", &messages);
            template.render("profile/view_user.html", &ctx_tmpl)
        }
        Err(_) => template.render_500("Erreur lors de la recherche"),
    }
}

/// Blog form
pub async fn blog_form(ctx: RuniqueContext, template: TemplateContext) -> Response {
    let form = BlogForm::build(ctx.engine.tera.clone());
    let ctx_tmpl = context! {
        "title" => "Formulaire Blog",
        "blog_form" => form
    };
    template.render("blog/blog.html", &ctx_tmpl)
}

/// Soumission blog
pub async fn soumission_blog(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<BlogForm>,
) -> Response {
    let db = ctx.engine.db.clone();

    if form.is_valid().await {
        match form.save(&*db).await {
            Ok(_) => {
                success!(ctx.flash => "Formulaire sauvegardé avec succès !");
                Redirect::to("/blog").into_response()
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
                let ctx_tmpl = context! {
                    "title" => "Erreur de base de données",
                    "blog_form" => form,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous")
                };
                template.render("blog/blog.html", &ctx_tmpl)
            }
        }
    } else {
        let ctx_tmpl = context! {
            "title" => "Erreur de retour",
            "blog_form" => form
        };
        template.render("blog/blog.html", &ctx_tmpl)
    }
}

/// Page "À propos"
pub async fn about(mut ctx: RuniqueContext, template: TemplateContext) -> Response {
    success!(ctx.flash => "Ceci est un message de succès.");
    info!(ctx.flash => "Ceci est un message d'information.");
    error!(ctx.flash => "Ceci est un message d'erreur.");
    warning!(ctx.flash => "Ceci est un message d'avertissement.");

    let ctx_tmpl = context! {
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera."
    };
    template.render("about/about.html", &ctx_tmpl)
}

/// Test CSRF - Handler simple pour tester la protection CSRF
pub async fn test_csrf(mut ctx: RuniqueContext, template: TemplateContext) -> Response {
    let ctx_tmpl = context! {
        "title" => "Test CSRF",
        "content" => "Si vous voyez ce message, le test CSRF a réussi."
    };
    info!(ctx.flash => "CSRF test réussi !");
    template.render("about/about.html", &ctx_tmpl)
}

/// Refresh CSRF Token - Retourne un nouveau token via JSON
pub async fn refresh_csrf_token(_ctx: RuniqueContext, template: TemplateContext) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "csrf_token": template.csrf_token
        })),
    )
        .into_response()
}
