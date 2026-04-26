//! `DynForm` trait: object-safe abstraction of Runique forms for admin dynamic dispatch.
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr};

use crate::forms::form::Forms;

/// Object-safe trait wrapping `RuniqueForm` for admin dynamic dispatch.
///
/// `RuniqueForm` has a `Sized` bound which makes it non-object-safe.
/// `DynForm` exposes only the methods necessary for the generic handler,
/// without a `Sized` bound, via `async_trait` for `Box<dyn DynForm>` compatibility.
#[async_trait]
pub trait DynForm: Send + Sync {
    /// Form validation (fields + clean)
    async fn is_valid(&mut self) -> bool;

    /// Database save with transaction
    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr>;

    /// Access to the underlying `Forms` for Tera rendering
    fn get_form(&self) -> &Forms;

    /// Mutable access to inject a DB error into the rendering
    fn get_form_mut(&mut self) -> &mut Forms;
}
