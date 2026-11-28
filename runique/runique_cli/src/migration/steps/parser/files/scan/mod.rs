//! Scanning the entities directory for `model!{}` and `extend!{}` declarations.
mod merge_extend_schemas;
mod scan_entities;
mod scan_extend_blocks;
mod scan_models;

use crate::migration::{
    steps::parser::files::{enums_of::enums_of, rust_files::rust_files},
    *,
};
use anyhow::{Context, Result};
use runique_i18n::tf;
use std::{fs, path::Path};

/// Tables created by `EihwazUsersMigration` + `AdminTableMigration`, never
/// migrated from the app's entities.
const FRAMEWORK_TABLES: &[&str] = &[
    "eihwaz_users",
    "eihwaz_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_users_groupes",
    "eihwaz_sessions",
    "eihwaz_reset_tokens",
];

/// What the entities directory declares: the tables to migrate, and the
/// tables of the models declared `managed: false`, list tables included.
pub struct ScannedEntities {
    pub schemas: Vec<ParsedSchema>,
    pub paused: Vec<String>,
    /// What the managed models declare with `renamed_from`: (former table,
    /// table). Declared only: which ones hold is decided against the snapshots.
    pub renames: Vec<(String, String)>,
}

pub use merge_extend_schemas::merge_extend_schemas;
pub use scan_entities::scan_entities;
pub use scan_extend_blocks::scan_extend_blocks;
pub use scan_models::scan_models;
