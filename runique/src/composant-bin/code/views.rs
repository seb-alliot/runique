use crate::forms::{LoginForm, SignupForm};
use runique::prelude::*;

/// Home page
pub async fn index(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "Welcome to Runique",
        "description" => "A web framework inspired by Django.",
    });
    request.render("index.html")
}

/// Sign up
pub async fn sign_up(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form();

    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "signup.html";

    // Not sent (first display): blank form, no message. Sent but invalid:
    // the same page again, each field with its errors; `save()` is never
    // attempted.
    let mut form = match ValidationForm::try_form(form, &request, "signup").await {
        Ok(validated) => validated.into_form(),
        Err(form) => {
            if !form.is_submitted() {
                context_update!(request => {
                    "title" => "Sign up",
                    "signup_form" => &form,
                });
                return request.render(template);
            }
            context_update!(request => {
                "title" => "Sign up",
                "signup_form" => &form,
                "messages" => flash_now!(error => "The form has errors. Please check the fields below."),
            });
            return request.render(template);
        }
    };

    match form.save(&request.engine.db).await {
        Ok(user) => {
            // `SignupForm::save` activates the account; a project that
            // confirms the email first gets `CannotSignIn` here until then.
            match login(&request.session, &user, None, false).await {
                Ok(()) => {
                    success!(request.notices => format!("Welcome {}!", user.username));
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
            form.forms_mut().database_error(&err);
            context_update!(request => {
                "title" => "Sign up",
                "signup_form" => &form,
                "messages" => flash_now!(error => "An error occurred while signing up. Please try again."),
            });
            request.render(template)
        }
    }
}

/// Sign in
pub async fn sign_in(mut request: Request) -> AppResult<Response> {
    let form: LoginForm = request.form();

    if request.user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "login.html";
    let validated = match ValidationForm::try_form(form, &request, "login").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => {
                "title" => "Sign in",
                "login_form" => &form,
            });
            return request.render(template);
        }
    };

    let username = validated.cleaned::<String>("username").unwrap_or_default();
    let password = validated.cleaned::<String>("password").unwrap_or_default();
    // Checks the password even for an unknown username (same timing either
    // way) and refuses an inactive or never activated account.
    if let Some(user) = authenticate_user(&request.engine.db, &username, &password).await
        && login(&request.session, &user, None, false).await.is_ok()
    {
        success!(request.notices => format!("Welcome back {}!", user.username));
        return Ok(Redirect::to("/").into_response());
    }

    context_update!(request => {
        "title" => "Sign in",
        "login_form" => &*validated,
        "messages" => flash_now!(error => "Invalid username or password."),
    });
    request.render(template)
}

/// Sign out. Runique checks a POST's CSRF token when the view reads its form;
/// this one reads none, so it checks the token itself, or another site could
/// sign the user out.
pub async fn sign_out(request: Request) -> AppResult<Response> {
    if !request.prisme.csrf_valid {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    // An error here means the session could not be closed: it is still alive,
    // so the user gets an error page rather than a false "signed out".
    logout(&request.session, None).await?;
    Ok(Redirect::to("/").into_response())
}

/// About page
pub async fn about(mut request: Request) -> AppResult<Response> {
    context_update!(request => {
        "title" => "About",
    });
    request.render("about.html")
}
