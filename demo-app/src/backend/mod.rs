pub mod auth;
pub mod blog;
mod cards;
pub mod comparatif;
pub mod contribution;
pub mod cours;
pub mod doc;
pub mod forms;
pub mod lang;
mod pages;
pub mod seeds;
pub mod upload_media;

pub use cards::*;
pub use pages::*;
pub use seeds::run_seeds;

use runique::prelude::*;

#[derive(serde::Serialize)]
pub struct FieldGroup {
    pub type_name: String,
    pub fields: Vec<crate::entities::form_field::Model>,
}

pub async fn inject_auth(request: &mut Request) {
    let user = is_authenticated(&request.session).await;
    request.context.insert("user", &user);
    if let Some(user) = &request.user {
        request.context.insert("username", &user.username);
    }
}

pub async fn inject_globals(request: &mut Request) {
    inject_auth(request).await;
    let user = &request.user;
    let release: Option<crate::entities::runique_release::Model> =
        search!(crate::entities::runique_release::Entity => desc Id)
            .first(&request.engine.db)
            .await
            .unwrap_or(None);

    // A language in the URL wins, so a shared link shows what it was shared for.
    let lang = match request.path_params.get("lang").map(|l| l.as_str()) {
        Some("fr") => "fr",
        Some("en") => "en",
        _ => lang::visitor_lang(&request.headers),
    };

    context_update!(request =>{
        "runique_release" => &release,
        "user" => user,
        "lang" => lang,
        "ui_lang" => lang,
    }
    )
}
