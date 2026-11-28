//! Snapshots: reading the previous state of a table, and upgrading old snapshots.
mod extend_snapshot_content;
mod extended_tables;
mod parse_create_file;
mod previous_snapshot;
mod read_all;
mod read_extend;
mod records_lengths;
mod snapshot_upgrades;

use crate::migration::{
    steps::parser::files::{
        rust_files::rust_files,
        scan::{merge_extend_schemas, scan_extend_blocks},
    },
    *,
};
use anyhow::{Context, Result};
use runique_i18n::tf;
use std::{fs, path::Path};

pub(crate) use extend_snapshot_content::extend_snapshot_content;
pub(crate) use extended_tables::extended_tables;
pub use parse_create_file::parse_create_file;
pub(crate) use previous_snapshot::previous_snapshot;
pub(crate) use read_all::read_all;
pub(crate) use read_extend::read_extend;
pub(crate) use records_lengths::records_lengths;
pub(crate) use snapshot_upgrades::snapshot_upgrades;
