use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};

use crate::forms::form::Forms;

/// Trait object-safe wrappant RuniqueForm pour le dispatch dynamique admin.
///
/// `RuniqueForm` a un bound `Sized` qui le rend non object-safe.
/// `DynForm` expose uniquement les méthodes nécessaires au handler générique,
/// sans bound `Sized`, via `async_trait` pour la compatibilité `Box<dyn DynForm>`.
#[async_trait]
pub trait DynForm: Send + Sync {
    /// Validation du formulaire (champs + clean)
    async fn is_valid(&mut self) -> bool;

    /// Sauvegarde en base avec transaction
    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr>;

    /// Accès au Forms sous-jacent pour le rendu Tera
    fn get_form(&self) -> &Forms;

    /// Accès mutable pour injecter une erreur DB dans le rendu
    fn get_form_mut(&mut self) -> &mut Forms;
}
