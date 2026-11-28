//! The `makemigrations` command: the master (`migration::master`) checks the project
//! and connects, then hands the run to the orchestrator (`orchestrator`), which goes through the steps (`steps`):
//! reads the models and the snapshots, compares them, checks what changed,
//! and writes the migration files in one batch.
use crate::{db::DatabaseEngine, migration::*};
use anyhow::Result;
use chrono::Utc;
use runique_i18n::{t, tf};
use std::fs;

use crate::migration::steps::{
    changes::report::{check_destructive, check_identifier_lengths, warn_unmigrated_type_changes},
    generate::{
        init_migrate,
        plan::{
            Plan, Stage, Stages, build_drop_plan, build_enum_plan, build_extend_plan,
            build_main_plan, commit_plan, replace_waiting, settled_migrations, waiting_migrations,
        },
    },
    parser::files::{
        scan::{merge_extend_schemas, scan_extend_blocks},
        snapshot::{self, snapshot_upgrades},
    },
};

mod orchestrator;
use crate::migration::master::BuilderMaster;
use orchestrator::BuilderOrchestrator;

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;

/// Entry point of the `makemigrations` command: scans models and `extend!{}`
/// blocks, diffs them against their last snapshot, blocks on destructive
/// changes unless `force` is set, then writes the full migration plan atomically.
///
/// A snapshot is the state of a table once `migration up` applied it: this
/// command never writes one. Run again before anything was applied, it starts
/// from the same snapshots and its result takes the place of the migrations
/// still waiting, instead of piling on them.
///
/// `engine` is the one the project's database runs on: the command is
/// connected to it before it gets here.
pub fn run(
    entities_path: &str,
    migrations_path: &str,
    force: bool,
    engine: DatabaseEngine,
) -> Result<()> {
    BuilderOrchestrator::new(entities_path, migrations_path, engine)
        .force(force)
        .read()?
        .compare()?
        .check()?
        .order()?
        .create_files()
        .build()
}

/// `makemigrations`: connects first, then plans and writes. An unreachable
/// database stops the command before anything is read or written, and so
/// does an engine in `.env` that the project is not compiled for.
pub async fn makemigrations(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    dotenvy::dotenv().ok();
    BuilderMaster::new()
        .force(force)
        .makemigrations(entities_path, migrations_path)
        .await
}
