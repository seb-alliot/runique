use crate::formulaire::{LoginForm, RegisterForm};
use runique::prelude::*;

async fn inject_auth(request: &mut Request) {
    let user = is_authenticated(&request.session).await;
    context_update!(request => {
        "user" => user,
    });
}

/// Page d'accueil
pub async fn index(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Welcome to Runique",
        "description" => "A web framework inspired by Django",
    });
    request.render("index.html")
}

/// Inscription
pub async fn soumission_inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "inscription_form.html";

    // GET (nothing submitted yet): blank form, no flash. Submitted but invalid
    // (POST, or PUT/DELETE/PATCH since `view!{}` registers all methods): shared
    // error render below, `save()` is never attempted.
    let mut form = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated.into_form(),
        Err(form) => {
            if request.method.is_safe() {
                context_update!(request => {
                    "title" => "Inscription",
                    "inscription_form" => &form,
                });
                return request.render(template);
            }
            context_update!(request => {
                "title" => "Inscription",
                "inscription_form" => &form,
                "messages" => flash_now!(error => "An error occurred while registering. Please try again."),
            });
            return request.render(template);
        }
    };

    match form.save(&request.engine.db).await {
        Ok(user) => {
            // `RegisterForm::save` activates the account; a project that
            // confirms the email first gets `CannotSignIn` here until then.
            match login(&request.session, &user, None, false).await {
                Ok(()) => {
                    success!(request.notices => format!("Welcome {} !", user.username));
                }
                Err(LoginError::CannotSignIn) => {
                    info!(request.notices => "Account created: it must be activated before you can sign in.");
                }
                Err(_) => {
                    warning!(request.notices => "Account created, but signing in failed: please sign in.");
                }
            }
            Ok(Redirect::to("/").into_response())
        }
        Err(err) => {
            form.get_form_mut().database_error(&err);
            context_update!(request => {
                "title" => "Inscription",
                "inscription_form" => &form,
                "messages" => flash_now!(error => "An error occurred while registering. Please try again."),
            });
            request.render(template)
        }
    }
}

/// Connexion
pub async fn connexion(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "login_form.html";
    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Sign in",
                "login_form" => &form,
            });
            return request.render(template);
        }
    };

    let username = validated.cleaned_string("username").unwrap_or_default();
    let password = validated.cleaned_string("password").unwrap_or_default();
    // Checks the password even for an unknown username (same timing either
    // way) and refuses an inactive or never activated account.
    if let Some(user) = authenticate_user(&request.engine.db, &username, &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        success!(request.notices => format!("Welcome back {} !", user.username));
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Sign in",
        "login_form" => &*validated,
        "messages" => flash_now!(error => "Invalid username or password."),
    });
    request.render(template)
}

/// About page
pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "About",
        "content" => "Runique is a web framework inspired by Django, built on Axum and Tera.",
    });
    request.render("about/about.html")
}
