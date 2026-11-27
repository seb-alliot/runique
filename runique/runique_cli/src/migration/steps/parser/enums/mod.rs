//! The enum as an object of its own: read from the models, where it is
//! declared once and used by columns, and read back from its snapshot.
mod of_schemas;
mod previous;
mod read_snapshots;
mod snapshot;

use crate::migration::helpers::schemas::db_columns;
use crate::migration::paths::{enum_snapshot_dir, enum_snapshot_file_path};
use crate::migration::steps::generate::generate_enum_snapshot_file;
use crate::migration::steps::parser::files::rust_files::rust_files;
use crate::migration::steps::parser::syn_extract::extract_alias_new_str;
use crate::migration::types::{EnumPlace, ParsedEnum, ParsedSchema};
use anyhow::Result;
use runique_i18n::tf;
use std::{fs, path::Path};
use syn::visit::Visit;

pub use of_schemas::enums_of_schemas;
pub use previous::{enum_snapshot_upgrades, previous_enums, with_enum_values};
pub use read_snapshots::read_enum_snapshots;
pub use snapshot::parse_enum_snapshot;
