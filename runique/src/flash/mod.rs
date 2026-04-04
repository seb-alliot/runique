//! Messages flash — stockage en session, niveaux (success/error/info/warning), extracteur Axum.
pub mod flash_manager;
pub mod flash_struct;

pub use flash_manager::Message;
pub use flash_struct::*;
