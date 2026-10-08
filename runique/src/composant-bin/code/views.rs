use crate::formulaire::RegisterForm;
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
            // New accounts start inactive (see `RegisterForm::save`): `login`
            // refuses them until they are activated.
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

/// About page
pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "About",
        "content" => "Runique is a web framework inspired by Django, built on Axum and Tera.",
    });
    request.render("about/about.html")
}
