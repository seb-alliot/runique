//! The migration plan: the files of a run, in the order they run, written in
//! one atomic commit. What changed is computed before, in `diff`, and
//! checked in `changes`.
mod build_drop_plan;
mod build_enum_plan;
mod build_extend_plan;
mod build_main_plan;
mod commit_plan;
mod replace_waiting;
mod settled_migrations;
mod stage;
mod stages;
mod waiting_migrate;
mod waiting_migrations;

use crate::atomic::{AtomicBuilder, SourceItems};
use crate::db::DatabaseEngine;
use crate::migration::steps::generate::lib_file::{
    lib_modules, planned_lib, strip_sea_orm_cli_placeholder_file,
};
use crate::migration::*;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use waiting_migrate::waiting_migrate;

pub(crate) use crate::migration::steps::generate::instruction::Order;
pub(crate) use build_drop_plan::build_drop_plan;
pub(crate) use build_enum_plan::build_enum_plan;
pub(crate) use build_extend_plan::build_extend_plan;
pub(crate) use build_main_plan::build_main_plan;
pub(crate) use commit_plan::commit_plan;
pub(crate) use replace_waiting::replace_waiting;
pub(crate) use settled_migrations::settled_migrations;
pub(crate) use stage::Stage;
pub(crate) use stages::Stages;
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
