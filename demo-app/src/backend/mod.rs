pub mod auth;
pub mod blog;
mod cards;
pub mod contribution;
pub mod cours;
pub mod doc;
pub mod forms;
mod pages;
pub mod seeds;

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
    let connected = is_authenticated(&request.session).await;
    let username = get_username(&request.session).await;
    request.context.insert("connected", &connected);
    request.context.insert("current_user", &username);
}

pub async fn inject_globals(request: &mut Request) {
    inject_auth(request).await;

    let release = crate::entities::runique_release::Entity::find()
        .order_by_desc(crate::entities::runique_release::Column::Id)
        .one(&*request.engine.db)
        .await
        .unwrap_or(None);

    if let Some(ref r) = release {
        request.context.insert("runique_release", r);
    }
}
