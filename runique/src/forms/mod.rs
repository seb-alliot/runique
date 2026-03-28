pub mod base;
pub mod extractor;
pub mod field;
pub mod fields;
pub mod form;
pub mod generic;
pub mod model_form;
pub mod options;
pub mod prisme;
pub mod renderer;
#[cfg(test)]
mod test_forms;
pub mod validator;

pub use base::*;
pub use extractor::*;
pub use field::*;
pub use fields::*;
pub use form::*;
pub use generic::*;
pub use model_form::*;
pub use options::*;
pub use prisme::*;
pub use renderer::*;
pub use validator::*;

/// Associe un formulaire à une entité SeaORM.
///
/// Généré automatiquement par `#[form(schema = ..., model = EntityPath)]`.
/// Permet d'utiliser le formulaire comme porte d'entrée des requêtes :
///
/// ```rust,ignore
/// search!(@UserForm => Username = "alice")
/// UserForm::objects.get(&db, id).await?
/// ```
pub trait FormEntity {
    type Entity: sea_orm::EntityTrait;
}
