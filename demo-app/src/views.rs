use crate::entities::blog::Entity as BlogEntity;
use crate::form_test::TestAllFieldsForm;
use crate::formulaire::*;
use runique::middleware::auth::login as auth_login;
use runique::prelude::user::Entity as UserEntity;
use runique::prelude::*;

// ─── Utilitaire : injecter l'état auth dans le contexte Tera ─────────────────
async fn inject_auth(request: &mut Request) {
    let connected = is_authenticated(&request.session).await;
    let username = get_username(&request.session).await;
    request.context.insert("connected", &connected);
    request.context.insert("current_user", &username);
}

// ───  Index  ─────────────────────────────────────────────────────────────────
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

// ─── Inscription ──────────────────────────────────────────────────────────────

pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }

    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription utilisateur",
            "inscription_form" => &form,
        });
        return request.render("inscription_form.html");
    }

    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                auth_login(&request.session, user.id, &user.username)
                    .await
                    .ok();
                success!(request.notices => format!("Bienvenue {} ! Votre compte est créé.", user.username));
                return Ok(Redirect::to("/profil").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
            }
        }
    }
    context_update!(request => {
        "title" => "Erreur de validation",
        "inscription_form" => &form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
    });
    request.render("inscription_form.html")
}

// ─── Connexion ────────────────────────────────────────────────────────────────

pub async fn login(mut request: Request, Prisme(form): Prisme<LoginForm>) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }

    let template = "auth/login.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Connexion",
            "login_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        let username_val = form.get_form().get_value("username").unwrap_or_default();
        let password_val = form.get_form().get_value("password").unwrap_or_default();

        if !username_val.is_empty() && !password_val.is_empty() {
            let db = request.engine.db.clone();
            let user_opt = UserEntity::find()
                .filter(runique::prelude::user::Column::Username.eq(&username_val))
                .one(&*db)
                .await
                .unwrap_or(None);

            match user_opt {
                Some(user) if user.is_active => {
                    if verify(&password_val, &user.password) {
                        auth_login(&request.session, user.id, &user.username)
                            .await
                            .ok();
                        success!(request.notices => format!("Bienvenue {} !", user.username));
                        return Ok(Redirect::to("/profil").into_response());
                    }
                }
                _ => {}
            }
        }

        flash_now!(error => "Identifiants invalides");
        context_update!(request => {
            "title" => "Connexion",
            "login_form" => &form,
            "auth_error" => &true,
        });
        return request.render(template);
    }

    request.render(template)
}

// ─── Déconnexion ─────────────────────────────────────────────────────────────

pub async fn deconnexion(request: Request) -> AppResult<Response> {
    logout(&request.session).await.ok();
    Ok(Redirect::to("/").into_response())
}

// ─── Profil ───────────────────────────────────────────────────────────────────

pub async fn profil(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if !is_authenticated(&request.session).await {
        warning!(request.notices => "Connectez-vous pour accéder à votre profil.");
        return Ok(Redirect::to("/login").into_response());
    }

    let user_id = get_user_id(&request.session).await;
    let username = get_username(&request.session).await;

    let user_opt = if let Some(id) = user_id {
        UserEntity::find_by_id(id)
            .one(&*request.engine.db)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    context_update!(request => {
        "title" => "Mon profil",
        "username" => &username,
        "profile_user" => &user_opt,
        "connected" => &true,
    });

    request.render("profile/profile.html")
}

// ─── Recherche utilisateur ────────────────────────────────────────────────────

pub async fn info_user(
    mut request: Request,
    Prisme(mut form): Prisme<UsernameForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "profile/view_user.html";

    if request.is_get() && form.is_valid().await {
        let db = request.engine.db.clone();

        // Get the username value from the form
        let username_val = form.get_form().get_value("username").unwrap_or_default();

        let user_opt = UserEntity::find()
            .filter(runique::prelude::user::Column::Username.eq(&username_val))
            .one(&*db)
            .await
            .unwrap_or(None);

        match user_opt {
            Some(user) => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "username" => &user.username,
                    "email" => &user.email,
                    "found_user" => &user,
                    "user" => &form,
                    "messages" => flash_now!(success => "Utilisateur trouvé !"),
                });
            }
            None => {
                context_update!(request => {
                    "title" => "Vue utilisateur",
                    "user" => &form,
                    "messages" => flash_now!(warning => "Utilisateur introuvable."),
                });
            }
        }

        request.render(template)
    } else {
        // Always return a response
        context_update!(request => {
            "title" => "Vue utilisateur",
            "user" => &form,
        });
        request.render(template)
    }
}

// ─── Blog ─────────────────────────────────────────────────────────────────────

pub async fn blog_list(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let articles = BlogEntity::find()
        .order_by_desc(crate::entities::blog::Column::Id)
        .all(&*request.engine.db)
        .await
        .unwrap_or_default();

    context_update!(request => {
        "title" => "Blog — Articles",
        "articles" => &articles,
    });

    request.render("blog/blog_list.html")
}

pub async fn blog_save(
    mut request: Request,
    Prisme(mut blog): Prisme<BlogForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "blog/blog.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Créer un article de blog",
            "blog_form" => &blog,
        });
        return request.render(template);
    }

    if request.is_post() {
        if blog.is_valid().await {
            match blog.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Article sauvegardé !");
                    return Ok(Redirect::to("/blog/liste").into_response());
                }
                Err(err) => {
                    blog.get_form_mut().database_error(&err);
                    context_update!(request => {
                        "title" => "Erreur base de données",
                        "blog_form" => &blog,
                    });
                    return request.render(template);
                }
            }
        }

        context_update!(request => {
            "title" => "Erreur de validation",
            "blog_form" => &blog,
            "messages" => flash_now!(error => "Veuillez corriger les erreurs ci-dessous"),
        });
        return request.render(template);
    }

    request.render(template)
}

pub async fn blog_detail(Path(id): Path<i32>, mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let article = BlogEntity::find_by_id(id)
        .one(&*request.engine.db)
        .await
        .unwrap_or(None);

    match article {
        Some(a) => {
            context_update!(request => {
                "title" => &a.title,
                "article" => &a,
            });
            request.render("blog/blog_detail.html")
        }
        None => {
            warning!(request.notices => "Article introuvable.");
            Ok(Redirect::to("/blog/liste").into_response())
        }
    }
}

// ─── À propos ────────────────────────────────────────────────────────────────

pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
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

// ─── CSRF test ────────────────────────────────────────────────────────────────

pub async fn test_csrf(request: Request) -> AppResult<Response> {
    success!(request.notices => "CSRF token validé avec succès !");
    Ok(Redirect::to("/").into_response())
}

// ─── Upload image ─────────────────────────────────────────────────────────────

pub async fn upload_image_submit(
    mut request: Request,
    Prisme(mut form): Prisme<ImageForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
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

// ─── Test des champs ─────────────────────────────────────────────────────────

pub async fn test_fields(
    mut request: Request,
    Prisme(mut form): Prisme<TestAllFieldsForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;
    let template = "forms/field_test.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Test des champs de formulaire Runique",
            "description" => "Page de test exhaustif de tous les types de champs",
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
