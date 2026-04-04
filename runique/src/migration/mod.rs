//! Module de migration — définitions DSL, schémas, colonnes, FK, index, hooks, relations et utilitaires.
//!
//! Le point d'entrée public est [`schema::ModelSchema`], source unique de vérité pour un modèle.
//! Les sous-modules exposent les briques élémentaires (colonnes, clés primaires, FK…) et les
//! utilitaires de diff/génération SQL utilisés par le moteur de migration.
pub mod column;
pub mod foreign_key;
pub mod hooks;
pub mod index;

pub mod primary_key;
pub mod relation;
pub mod schema;
pub mod utils;

// Re-exports
pub use column::*;
pub use foreign_key::*;
pub use hooks::*;
pub use index::*;
pub use primary_key::*;
pub use relation::*;
pub use schema::*;
pub use utils::*;
