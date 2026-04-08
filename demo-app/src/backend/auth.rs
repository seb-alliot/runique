use crate::formulaire::{LoginForm, RegisterForm};
use runique::middleware::auth::login as auth_login;
use runique::prelude::user::ActiveModel as UserActiveModel;
use runique::prelude::user::Entity as UserEntity;
use runique::prelude::*;

pub async fn register_user(
    form: &RegisterForm,
    db: &sea_orm::DatabaseConnection,
) -> Result<runique::prelude::user::Model, sea_orm::DbErr> {
    form.save(db).await
}

pub async fn authenticate_user(
    db: &sea_orm::DatabaseConnection,
    username: &str,
    password: &str,
) -> Option<runique::prelude::user::Model> {
    let user = find_user_by_username(db, username).await?;
    if user.is_active && verify(password, &user.password) {
        Some(user)
    } else {
        None
    }
}

pub async fn find_user_by_username(
    db: &sea_orm::DatabaseConnection,
    username: &str,
) -> Option<runique::prelude::user::Model> {
    // Construit la requête
    let query = search!(UserEntity => Username eq username.trim());
    let result = query.first(db).await;
    result.unwrap_or(None)
}

pub async fn find_user_by_id(
    db: &sea_orm::DatabaseConnection,
    id: runique::utils::pk::Pk,
) -> Option<runique::prelude::user::Model> {
    UserEntity::find_by_id(id).one(db).await.unwrap_or(None)
}

pub async fn handle_inscription(
    request: &mut Request,
    form: &mut RegisterForm,
    headers: &HeaderMap,
) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/inscription.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("inscription", &db).await;
    if request.is_get() {
        context_update!(request => {
            "title"            => "User registration",
            "inscription_form" => &*form,
            "code_examples"    => &code_examples,
            "doc_links"        => &doc_links,
        });
        return request.render(template);
    }
    if request.is_post() {
        match register_user(form, &request.engine.db).await {
            Ok(user) => {
                let token = reset_token::generate(&user.email);
                let encrypted = reset_token::encrypt_email(&token, &user.email);
                let base_url = headers
                    .get("host")
                    .and_then(|v| v.to_str().ok())
                    .map(|h| format!("http://{h}"))
                    .unwrap_or_else(|| "http://localhost:3000".to_string());
                let activate_url = format!("{}/activate/{}/{}", base_url, token, encrypted);
                if mailer_configured() {
                    Email::new()
                        .to(user.email.clone())
                        .subject("Activez votre compte")
                        .html(format!(
                            "<p>Bonjour {},</p><p>Cliquez sur ce lien pour activer votre compte :</p><p><a href=\"{}\">Activer mon compte</a></p>",
                            user.username, activate_url
                        ))
                        .send()
                        .await
                        .ok();
                }
                success!(request.notices => "Compte créé ! Consultez votre email pour l'activer.");
                return Ok(Redirect::to("/login").into_response());
            }
            Err(err) => form.get_form_mut().database_error(&err),
        }
    }
    context_update!(request => {
        "title"            => "Validation error",
        "inscription_form" => &*form,
        "code_examples"    => &code_examples,
        "doc_links"        => &doc_links,
        "messages"         => flash_now!(error => "Please correct the errors"),
    });
    request.render(template)
}

pub async fn handle_activate(
    request: &mut Request,
    token: String,
    encrypted_email: String,
) -> AppResult<Response> {
    // Vérifier et décrypter avant de consommer (consume est irréversible)
    if !reset_token::peek(&token) {
        warning!(request.notices => "Lien d'activation invalide ou expiré.");
        return Ok(Redirect::to("/login").into_response());
    }

    let Some(email) = reset_token::decrypt_email(&token, &encrypted_email) else {
        warning!(request.notices => "Lien d'activation invalide.");
        return Ok(Redirect::to("/login").into_response());
    };

    // Consommer le token (usage unique)
    reset_token::consume(&token);

    let db = request.engine.db.clone();

    // Trouver l'utilisateur par email
    let query = search!(UserEntity => Email eq email.trim());
    let Some(user) = query.first(&db).await.unwrap_or(None) else {
        warning!(request.notices => "Compte introuvable.");
        return Ok(Redirect::to("/login").into_response());
    };

    // Activer le compte
    let active_model = UserActiveModel {
        id: Set(user.id),
        is_active: Set(true),
        ..Default::default()
    };
    if active_model.update(&*db).await.is_err() {
        warning!(request.notices => "Erreur lors de l'activation.");
        return Ok(Redirect::to("/login").into_response());
    }

    // Connecter directement
    auth_login(
        &request.session,
        &db,
        user.id,
        &user.username,
        false,
        false,
        None,
        false,
    )
    .await
    .ok();

    success!(request.notices => format!("Bienvenue {} ! Votre compte est activé.", user.username));
    Ok(Redirect::to("/profil").into_response())
}

pub async fn handle_login(request: &mut Request, form: &mut LoginForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/login.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("login", &db).await;
    if request.is_get() {
        context_update!(request => {
            "title"         => "Login",
            "login_form"    => form,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
        });
        return request.render(template);
    }
    let credentials = get_credentials(form);
    if request.is_post() && form.is_valid().await {
        if let Some((username_val, password_val)) = &credentials
            && let Some(user) =
                authenticate_user(&request.engine.db, username_val, password_val).await
        {
            auth_login(
                &request.session,
                &request.engine.db,
                user.id,
                &user.username,
                user.is_staff,
                user.is_superuser,
                None,
                false,
            )
            .await
            .ok();
            success!(request.notices => format!("Welcome {}!", user.username));
            return Ok(Redirect::to("/profil").into_response());
        }
        context_update!(request => {
            "title"         => "Login",
            "login_form"    => form,
            "auth_error"    => &true,
            "code_examples" => &code_examples,
            "doc_links"     => &doc_links,
            "messages"      => flash_now!(error => "Invalid credentials"),
        });
        return request.render(template);
    }
    context_update!(request => {
        "title"         => "Login",
        "login_form"    => form,
        "code_examples" => &code_examples,
        "doc_links"     => &doc_links,
    });
    request.render(template)
}

pub fn get_credentials(form: &LoginForm) -> Option<(String, String)> {
    let u = form.get_form().get_value("username").unwrap_or_default();
    let p = form.get_form().get_value("password").unwrap_or_default();
    if u.is_empty() || p.is_empty() {
        None
    } else {
        Some((u, p))
    }
}

pub async fn get_profile_user(
    user_id: Option<runique::utils::pk::Pk>,
    db: &sea_orm::DatabaseConnection,
) -> Option<runique::prelude::user::Model> {
    find_user_by_id(db, user_id?).await
}
