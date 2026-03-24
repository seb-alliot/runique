pub mod auth;
pub mod blog;
mod cards;
pub mod contribution;
pub mod cour_seed;
pub mod cours;
pub mod doc;
pub mod doc_seed;
pub mod forms;
pub mod ia_seed;
mod pages;

pub use cards::*;
pub use pages::*;

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
