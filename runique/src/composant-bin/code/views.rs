use crate::formulaire::RegisterForm;
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
    inject_auth(&mut request).await;

    if is_authenticated(&request.session).await {
        return Ok(Redirect::to("/").into_response());
    }
    let template = "auth/inscription.html";
    if request.is_get() {
        context_update!(request => {
            "title" => "Inscription utilisateur",
            "inscription_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                auth_login(&request.session, user.id, &user.username)
                    .await
                    .ok();
                success!(request.notices => format!("Bienvenue {} ! Votre compte est créé.", user.username));
                return Ok(Redirect::to("/").into_response());
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
