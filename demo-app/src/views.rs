use crate::entities::users::Entity as UserEntity;
use crate::form_test::TestAllFieldsForm;
use crate::formulaire::*;
use runique::prelude::*;

/// Page d'accueil
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
        "status" => "Framework en cours de développement...",
        "backend" => "Axum pour le backend",
        "template" => "Tera comme moteur de templates",
        "tokio" => "Runtime asynchrone tokio",
        "session" => "Tower pour la gestion des sessions",
    });

    request.render("index.html")
}

/// Soumission du formulaire d'inscription
pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription utilisateur",
            "inscription_form" => &form,
        });
        return request.render("inscription_form.html");
    }

    if request.is_post() {
        if form.is_valid().await {
            let user = form.save(&request.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            println!("Nouvel utilisateur créé donné dans views.rs : {:?}", user);

            success!(request.notices => format!("Bienvenue {}, votre compte est créé !", user.username));
            return Ok(Redirect::to("/").into_response());
        }

        // Validation échouée
        context_update!(request => {
            "title" => "Erreur de validation",
            "inscription_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render("inscription_form.html");
    }

    // Cas fallback
    request.render("inscription_form.html")
}

/// Recherche d'utilisateur / vue utilisateur
pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    let template = "profile/view_user.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Rechercher un utilisateur",
            "user" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if !form.is_valid().await {
            context_update!(request => {
                "title" => "Rechercher un utilisateur",
                "user" => &form,
                "messages" => flash_now!(error => "Erreur de validation"),
            });
            return request.render(template);
        }

        let username = form.get_form().get_value("username").unwrap_or_default();
        let db = request.engine.db.clone();

        let user_opt = UserEntity::objects
            .filter(crate::formulaire::user::Column::Username.eq(&username))
            .first(&db)
            .await?;

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,
                    "user" => &form,
                    "messages" => flash_now!(success => "Voici les infos que tu voulais !"),
                });
            }
            None => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "user" => &form,
                    "messages" => flash_now!(warning => "Utilisateur introuvable. Inscrivez-vous !"),
                });
            }
        }

        return request.render(template);
    }

    request.render(template)
}

/// Blog save
pub async fn blog_save(
    mut request: Request,
    Prisme(mut form): Prisme<BlogForm>,
) -> AppResult<Response> {
    let template = "blog/blog.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Créer un article de blog",
            "blog_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            match form.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Article de blog sauvegardé avec succès !");
                    return Ok(Redirect::to("/").into_response());
                }
                Err(err) => {
                    form.get_form_mut().database_error(&err);
                    context_update!(request => {
                        "title" => "Erreur de base de données",
                        "blog_form" => &form,
                        "messages" => flash_now!(warning => "Veuillez corriger les erreurs ci-dessous"),
                    });
                    return request.render(template);
                }
            }
        }

        // Validation échouée
        context_update!(request => {
            "title" => "Erreur de validation",
            "blog_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
        });
        return request.render(template);
    }

    request.render(template)
}

/// Page "À propos"
pub async fn about(mut request: Request) -> AppResult<Response> {
    success!(request.notices => "Ceci est un message de succès.");
    info!(request.notices => "Ceci est un message d'information.");
    warning!(request.notices => "Ceci est un message d'avertissement.");
    error!(request.notices => "Ceci est un message d'erreur.");

    context_update!(request => {
        "title" => "À propos du Framework Runique",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });

    request.render("about/about.html")
}

/// Test CSRF
pub async fn test_csrf(request: Request) -> AppResult<Response> {
    success!(request.notices => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}

/// Upload image
pub async fn upload_image_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    let template = "forms/upload_image.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Uploader un fichier",
            "image_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Fichier uploadé avec succès !");
            return Ok(Redirect::to("/").into_response());
        }

        context_update!(request => {
            "title" => "Erreur de validation",
            "image_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
        });
        return request.render(template);
    }

    request.render(template)
}

/// Test des champs Runique
pub async fn test_fields(
    mut request: Request,
    Prisme(mut form): Prisme<TestAllFieldsForm>,
) -> AppResult<Response> {
    let template = "forms/field_test.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Test des champs de formulaire Runique",
            "description" => "Page de test des champs de formulaire Runique",
            "test_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            success!(request.notices => "Formulaire validé avec succès !");
            return Ok(Redirect::to("/test-fields").into_response());
        }

        context_update!(request => {
            "title" => "Erreur de validation",
            "test_form" => &form,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
        });
        return request.render(template);
    }

    request.render(template)
}
