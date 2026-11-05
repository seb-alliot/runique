use crate::backend::lang::msg;
use crate::formulaire::{LoginForm, RegisterForm};
use runique::prelude::runique_users::Entity as UserEntity;
use runique::prelude::*;

pub async fn register_user(
    form: &RegisterForm,
    db: &ADb,
) -> Result<runique::prelude::runique_users::Model, sea_orm::DbErr> {
    form.save(db).await
}

#[allow(dead_code)]
pub async fn find_user_by_username(
    db: &ADb,
    username: &str,
) -> Option<runique::prelude::runique_users::Model> {
    // Build the query
    let query = search!(UserEntity => Username eq username.trim());
    let result = query.first(db).await;
    result.unwrap_or(None)
}

/* /// One query for both searches; the rows are sorted back to each form in Rust.
pub async fn find_users_by_username(
    db: &ADb,
    username: Option<&str>,
    email: Option<&str>,
) -> (
    Option<runique::prelude::runique_users::Model>,
    Option<runique::prelude::runique_users::Model>,
) {
    let username = username.map(str::trim);
    let email = email.map(str::trim);
    let query = match (username, email) {
        (Some(name), Some(mail)) => search!(UserEntity => or(Username eq name, Email eq mail)),
        (Some(name), None) => search!(UserEntity => Username eq name),
        (None, Some(mail)) => search!(UserEntity => Email eq mail),
        (None, None) => return (None, None),
    };
    let rows = query.all(db).await.unwrap_or_default();
    let by_username =
        username.and_then(|name| rows.iter().find(|user| user.username == name).cloned());
    let by_email = email.and_then(|mail| rows.into_iter().find(|user| user.email == mail));
    (by_username, by_email)
} */

pub async fn handle_inscription(request: &mut Request, form: RegisterForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/inscription.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("inscription", &db).await;

    let mut validated = match ValidationForm::try_form(form, request).await {
        Ok(validated) => validated,
        Err(mut form) => {
            crate::backend::lang::localize_errors(form.forms_mut(), &request.headers);
            // Not sent (first display): blank form, no flash. Sent but
            // invalid: re-render with the validation-error flash.
            if !form.is_submitted() {
                context_update!(request => {
                    "title"            => "User registration",
                    "inscription_form" => &form,
                    "code_examples"    => &code_examples,
                    "doc_links"        => &doc_links,
                });
            } else {
                context_update!(request => {
                    "title"            => "Validation error",
                    "inscription_form" => &form,
                    "code_examples"    => &code_examples,
                    "doc_links"        => &doc_links,
                    "messages"         => flash_now!(error => msg(&request.headers, "Please correct the errors", "Veuillez corriger les erreurs")),
                });
            }
            return request.render(template);
        }
    };

    match register_user(&validated, &request.engine.db).await {
        Ok(user) => {
            let token = reset_token::generate(
                &request.engine.db,
                user.id,
                std::time::Duration::from_secs(86_400),
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to create account activation token");
                Box::new(AppError::new(ErrorContext::generic(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to create account activation token",
                )))
            })?;
            // Never the request's Host outside debug: a forged Host would send
            // the activation link — and its token — elsewhere.
            let Some(base_url) = request.public_url() else {
                return Err(Box::new(AppError::new(ErrorContext::generic(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "the public URL is not set (.with_public_url())",
                ))));
            };
            let activate_url = format!("{}/activate/{}", base_url, token);
            let (subject, body) = match crate::backend::lang::visitor_lang(&request.headers) {
                "fr" => (
                    "Activez votre compte",
                    format!(
                        "<p>Bonjour {},</p><p>Cliquez sur ce lien pour activer votre compte :</p><p><a href=\"{}\">Activer mon compte</a></p>",
                        user.username, activate_url
                    ),
                ),
                _ => (
                    "Activate your account",
                    format!(
                        "<p>Hello {},</p><p>Click this link to activate your account:</p><p><a href=\"{}\">Activate my account</a></p>",
                        user.username, activate_url
                    ),
                ),
            };
            if mailer_configured() {
                Email::new()
                    .to(user.email.clone())
                    .subject(subject)
                    .html(body)
                    .send()
                    .await
                    .ok();
            }
            success!(request.notices => msg(&request.headers, "Account created! Check your email to activate it.", "Compte créé ! Consultez vos mails pour l'activer."));
            return Ok(Redirect::to("/login").into_response());
        }
        Err(err) => validated.database_error(&err),
    }

    context_update!(request => {
        "title"            => "Validation error",
        "inscription_form" => &*validated,
        "code_examples"    => &code_examples,
        "doc_links"        => &doc_links,
        "messages"         => flash_now!(error => msg(&request.headers, "Please correct the errors", "Veuillez corriger les erreurs")),
    });
    request.render(template)
}

pub async fn handle_activate(request: &mut Request, token: String) -> AppResult<Response> {
    let Some(user_id) = reset_token::consume(&request.engine.db, &token).await else {
        warning!(request.notices => msg(&request.headers, "This activation link is invalid or has expired.", "Ce lien d'activation est invalide ou a expiré."));
        return Ok(Redirect::to("/login").into_response());
    };

    let db = request.engine.db.clone();

    // Activate the account — only a pending one: a blocked account stays blocked.
    let activated = match BuiltinUserEntity::activate_account(&db, user_id).await {
        Ok(Some(activated)) => activated,
        Ok(None) => {
            warning!(request.notices => msg(&request.headers, "This account is already activated or has been blocked.", "Ce compte est déjà activé ou a été bloqué."));
            return Ok(Redirect::to("/login").into_response());
        }
        Err(_) => {
            warning!(request.notices => msg(&request.headers, "Something went wrong while activating your account.", "Un problème est survenu pendant l'activation de votre compte."));
            return Ok(Redirect::to("/login").into_response());
        }
    };

    // Directly log in, with the account as activation left it.
    if login(&request.session, &activated, None, false)
        .await
        .is_err()
    {
        warning!(request.notices => msg(&request.headers, "Your account is active: please sign in.", "Votre compte est actif : connectez-vous."));
        return Ok(Redirect::to("/login").into_response());
    }

    let welcome_active = match crate::backend::lang::visitor_lang(&request.headers) {
        "fr" => format!(
            "Bienvenue {} ! Votre compte est maintenant actif.",
            activated.username
        ),
        _ => format!(
            "Welcome {}! Your account is now active.",
            activated.username
        ),
    };
    success!(request.notices => welcome_active);
    Ok(Redirect::to("/profil").into_response())
}

pub async fn handle_login(request: &mut Request, form: LoginForm) -> AppResult<Response> {
    crate::backend::inject_globals(request).await;
    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/profil").into_response());
    }
    let template = "auth/login.html";
    let db = request.engine.db.clone();
    let (code_examples, doc_links) = crate::backend::fetch_page_examples("login", &db).await;

    let validated = match ValidationForm::try_form(form, request).await {
        Ok(validated) => validated,
        Err(mut form) => {
            crate::backend::lang::localize_errors(form.forms_mut(), &request.headers);
            // No flash here: a global error (e.g. CSRF) is already rendered
            // inline by `{% form.login_form %}` (Forms::render() puts global
            // errors first) — a flash would just repeat the same text.
            context_update!(request => {
                "title"         => "Login",
                "login_form"    => &form,
                "code_examples" => &code_examples,
                "doc_links"     => &doc_links,
            });
            return request.render(template);
        }
    };

    let credentials = get_credentials(&validated);
    if let Some((username_val, password_val)) = &credentials
        && let Some(user) = authenticate_user(&request.engine.db, username_val, password_val).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        let welcome = match crate::backend::lang::visitor_lang(&request.headers) {
            "fr" => format!("Bienvenue {} !", user.username),
            _ => format!("Welcome {}!", user.username),
        };
        success!(request.notices => welcome);
        return Ok(Redirect::to("/profil").into_response());
    }

    context_update!(request => {
        "title"         => "Login",
        "login_form"    => &*validated,
        "auth_error"    => &true,
        "code_examples" => &code_examples,
        "doc_links"     => &doc_links,
        "messages"      => flash_now!(error => msg(&request.headers, "Invalid credentials", "Identifiants invalides")),
    });
    request.render(template)
}

pub fn get_credentials(form: &LoginForm) -> Option<(String, String)> {
    let u = form.cleaned::<String>("username")?;
    let p = form.cleaned::<String>("password")?;
    Some((u, p))
}

pub async fn get_profile_user(
    user_id: Option<runique::utils::pk::Pk>,
    username: Option<&str>,
    db: &ADb,
) -> (
    Option<runique::prelude::runique_users::Model>,
    Option<runique::prelude::runique_users::Model>,
) {
    let (Some(id), Some(name)) = (user_id, username.map(str::trim)) else {
        return (None, None);
    };
    let rows = search!(UserEntity => or(Id eq id, Username eq name))
        .all(db)
        .await
        .unwrap_or_default();
    let by_id = rows.iter().find(|user| user.id == id).cloned();
    let by_username = rows.into_iter().find(|user| user.username == name);
    (by_id, by_username)
}
