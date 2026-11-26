//! The migration plan: computing every change, then writing it in one atomic commit.
mod build_drop_plan;
mod build_extend_plan;
mod build_main_plan;
mod check_parallel;
mod commit_plan;
mod compute_main_changes;
mod creation_of;
mod cycle_keys;
mod dropped_tables;
mod effective_renames;
mod plan_extend_changes;
mod replace_waiting;
mod settled_migrations;
mod waiting_migrate;
mod waiting_migrations;

use crate::atomic::{AtomicBuilder, SourceItems};
use crate::migration::steps::generate::lib_file::{
    lib_modules, planned_lib, strip_sea_orm_cli_placeholder_file,
};
use crate::migration::steps::parser::files::scan::{merge_extend_schemas, scan_extend_blocks};
use crate::migration::steps::parser::files::snapshot::{extended_tables, previous_snapshot};
use crate::migration::*;
use anyhow::Result;
use runique_i18n::tf;
use std::fs;
use std::path::{Path, PathBuf};

use creation_of::creation_of;
use waiting_migrate::waiting_migrate;

pub(crate) use build_drop_plan::build_drop_plan;
pub(crate) use build_extend_plan::build_extend_plan;
pub(crate) use build_main_plan::build_main_plan;
pub(crate) use check_parallel::check_parallel;
pub(crate) use commit_plan::commit_plan;
pub(crate) use compute_main_changes::compute_main_changes;
pub(crate) use cycle_keys::cycle_keys;
pub(crate) use dropped_tables::dropped_tables;
pub(crate) use effective_renames::effective_renames;
pub(crate) use plan_extend_changes::plan_extend_changes;
pub(crate) use replace_waiting::replace_waiting;
pub(crate) use settled_migrations::settled_migrations;
pub(crate) use waiting_migrations::waiting_migrations;

/// A fully-computed migration plan: everything to create/write/register, no side effects.
/// Built before any IO so the destructive guard and a single atomic commit can run on it.
#[derive(Default)]
pub(crate) struct Plan {
    /// (path, content) files to write
    pub(crate) files: Vec<(String, String)>,
    /// migration modules to register in `lib.rs`, in order
    pub(crate) lib_modules: Vec<String>,
    /// waiting migrations this run replaces: their file is removed and their
    /// module taken out of `lib.rs`
    pub(crate) dropped_modules: Vec<String>,
}
