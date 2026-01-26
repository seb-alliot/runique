use crate::forms::{Blog as BlogForm, RegisterForm, UsernameForm};

// use crate::models::model_derive;
use crate::models::users::{self, Entity as UserEntity};

use runique::forms::Prisme;
use runique::prelude::*;

/// Page d'accueil
pub async fn index(mut template: TemplateContext) -> AppResult<Response> {
    context_update!(template => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "Axum pour le backend",
        "template" => "Tera comme moteur de templates",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions",
    });

    template.render("index.html")
}

// /// Formulaire d'inscription
pub async fn inscription(mut template: TemplateContext) -> AppResult<Response> {
    let form = template.form::<RegisterForm>();
    context_update!(template => {
        "title" => "Inscription utilisateur",
        "inscription_form" => &form,
    });

    template.render("inscription_form.html")
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut template: TemplateContext,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let db = template.engine.db.clone();

    if form.is_valid().await {
        match form.save(&db).await {
            Ok(user) => {
                success!(template.notices => format!("Bienvenue {}, votre compte a été créé !", user.username));
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                // Extraire automatiquement l'erreur d'unicité et l'assigner au bon champ
                form.get_form_mut().database_error(&err);

                context_update!(template => {
                    "title" => "Erreur de base de données",
                    "inscription_form" => &form,
                });

                return template.render("inscription_form.html");
            }
        }
    }

    context_update!(template => {
        "title" => "Erreur de validation",
        "inscription_form" => &form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
    });

    template.render("inscription_form.html")
}

// / Formulaire de recherche d'utilisateur
pub async fn search_user_form(mut template: TemplateContext) -> AppResult<Response> {
    let form = template.form::<UsernameForm>();

    context_update!(template => {
        "title" => "Rechercher un utilisateur",
        "user" => &form,
    });

    template.render("profile/view_user.html")
}

/// Exemple pour chercher un utilisateur
pub async fn info_user(
    mut template: TemplateContext,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    if !form.is_valid().await {
        // Retourner le formulaire avec les erreurs
        context_update!(template => {
            "title" => "Rechercher un utilisateur",
            "user" => &form,
            "messages" => flash_now!(error => "Erreur de validation"),
        });
        return template.render("profile/view_user.html");
    }

    let username = form.get_form().get_value("username").unwrap_or_default();
    let db = template.engine.db.clone();

    let user = UserEntity::objects
        .filter(users::Column::Username.eq(&username))
        .first(&db)
        .await?;

    match user {
        Some(user) => {
            context_update!(template => {
                "title" => "Vue utilisateur",
                "username" => &user.username,
                "email" => &user.email,
                "user" => &user,
                "messages" => flash_now!(success => "Voici les infos que tu voulais !"),
            });

            template.render("profile/view_user.html")
        }
        None => {
            context_update!(template => {
                "title" => "Vue utilisateur",
                "user" => &form,
                "messages" => flash_now!(warning => "Tu n'existe pas dans ma bdd !! Une honte je te le dis, inscris toi de suite !! "),
            });

            template.render("profile/view_user.html")
        }
    }
}

/// Blog form
pub async fn blog_form(mut template: TemplateContext) -> AppResult<Response> {
    let form = BlogForm::build(
        template.engine.tera.clone(),
        template.csrf_token.masked().as_str(),
    );

    context_update!(template => {
        "title" => "Créer un article de blog",
        "blog_form" => &form,
    });

    template.render("blog/blog.html")
}

/// Blag save
pub async fn blog_save(
    mut template: TemplateContext,
    Prisme(mut blog_save): Prisme<BlogForm>,
) -> AppResult<Response> {
    if blog_save.is_valid().await {
        match blog_save.save(&template.engine.db).await {
            Ok(_post) => {
                success!(template.notices => "Article de blog sauvegardé avec succès !");
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                blog_save.get_form_mut().database_error(&err);
                context_update!(template => {
                    "title" => "Erreur de base de données",
                    "blog_form" => &blog_save,
                    "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous"),
                });

                return template.render("blog/blog.html");
            }
        }
    }
    success!(template.notices => "Article de blog sauvegardé avec succès !");
    Ok(Redirect::to("/").into_response())
}

/// Page "À propos"
pub async fn about(mut template: TemplateContext) -> AppResult<Response> {
    success!(template.notices => "Ceci est un message de succès.");
    info!(template.notices => "Ceci est un message d'information.");
    warning!(template.notices => "Ceci est un message d'avertissement.");
    error!(template.notices => "Ceci est un message d'erreur.");

    context_update!(template => {
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });

    template.render("about/about.html")
}

/// Teste Csrf
pub async fn test_csrf(template: TemplateContext) -> AppResult<Response> {
    success!(template.notices => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}
