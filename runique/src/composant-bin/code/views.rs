use crate::formulaire::RegisterForm;
use runique::prelude::*;

async fn inject_auth(request: &mut Request) {
    let connected = is_authenticated(&request.session).await;
    let username = get_username(&request.session).await;
    request.context.insert("connected", &connected);
    request.context.insert("current_user", &username);
}

/// Page d'accueil
pub async fn index(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "Bienvenue sur Runique",
        "description" => "Un framework web moderne inspiré de Django",
    });
    request.render("index.html")
}

/// Inscription
pub async fn soumission_inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }

    let template = "inscription_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription",
            "inscription_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                auth_login(&request.session, user.id, &user.username)
                    .await
                    .ok();
                success!(request.notices => format!("Bienvenue {} !", user.username));
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                form.get_form_mut().database_error(&err);
            }
        }
    }

    context_update!(request => {
        "title" => "Inscription",
        "inscription_form" => &form,
        "messages" => flash_now!(error => "Veuillez corriger les erreurs"),
    });
    request.render(template)
}

/// Page "À propos"
pub async fn about(mut request: Request) -> AppResult<Response> {
    inject_auth(&mut request).await;
    context_update!(request => {
        "title" => "À propos",
        "content" => "Runique est un framework web inspiré de Django, construit sur Axum et Tera.",
    });
    request.render("about/about.html")
}
