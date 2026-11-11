//! Migration commands' logic: `makemigrations` (`run`, generates SeaORM migration
//! files from the `model!{}` DSL), `migration up/down/status` (`migrate`) and the
//! project's `migration` crate (`init_migrate`). The commands themselves are
//! called through `cli::makemigration` and `cli::migrate`.
mod lib_file;
pub mod migrate;
pub(crate) mod init_migrate;
mod plan;
mod report;
mod scan;
mod snapshot;
#[cfg(test)]
mod tests;

use crate::migration::*;
use crate::utils::trad::{t, tf};
use anyhow::Result;
use chrono::Utc;
use std::fs;

pub use crate::utils::*;

// Logic that lives in `migration/`, offered here too: the integration tests
// reach the whole command from this one module.
pub use crate::migration::{
    collect_destructive_messages, seaorm_alter_file_path, seaorm_alter_module_name,
    seaorm_extend_file_path, seaorm_extend_module_name,
};
pub(crate) use lib_file::initial_lib;
pub use lib_file::{ensure_admin_migration_positioned, update_migration_lib};
pub use scan::{merge_extend_schemas, scan_entities, scan_extend_blocks};
pub use snapshot::parse_create_file;
// Only the pipeline tests (`migration/tests_pipeline.rs`) reach it from outside.
#[cfg(test)]
pub(crate) use snapshot::previous_snapshot;

use plan::{
    Plan, build_extend_plan, build_main_plan, commit_plan, compute_main_changes,
    plan_extend_changes,
};
use report::{check_destructive, check_identifier_lengths, warn_unmigrated_type_changes};
use snapshot::snapshot_upgrades;

/// Entry point of the `makemigrations` command: scans models and `extend!{}`
/// blocks, diffs them against their last snapshot, blocks on destructive
/// changes unless `force` is set, then writes the full migration plan atomically.
pub fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    let schemas = scan_entities(entities_path)?;
    init_migrate::ensure(migrations_path)?;

    fs::create_dir_all(snapshot_dir(migrations_path))?;

    // ── Plan everything up front — nothing is written until the full plan
    //    (main models + extend!{} blocks) is computed and validated.
    let mut main_changes = compute_main_changes(&schemas, migrations_path)?;
    let extend_planned = plan_extend_changes(entities_path, migrations_path)?;

    let upgrades = snapshot_upgrades(
        entities_path,
        &schemas,
        &main_changes,
        &extend_planned,
        migrations_path,
    )?;

    if main_changes.is_empty() && extend_planned.is_empty() {
        for (path, content) in &upgrades {
            fs::write(path, content)?;
        }
        // A project without a model of its own still needs the framework's
        // tables (accounts, sessions): wired into `lib.rs` even then, and the
        // `sea-orm-cli migrate init` placeholder dropped.
        ensure_admin_migration_positioned(migrations_path)?;
        if !upgrades.is_empty() {
            println!(
                "{}",
                tf("makemigrations.snapshots_upgraded", &[upgrades.len()])
            );
        }
        // Silent, a run looked like it did nothing at all; the framework's
        // tables may still be waiting to be applied.
        println!("{}", t("makemigrations.no_changes"));
        println!("{}", t("makemigrations.apply_hint"));
        return Ok(());
    }

    // ── Single destructive guard over main + extend (honors --force) ──────
    let mut destructive_set: Vec<Changes> = main_changes.clone();
    destructive_set.extend(extend_planned.iter().map(|(_, c)| c.clone()));
    check_destructive(&destructive_set, force)?;
    check_identifier_lengths(&destructive_set)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    // referenced tables created before those referencing them
    main_changes = topological_sort_changes(main_changes);

    // ── Build a single unified plan (no writing) ──────────────────────────
    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &main_changes,
        &schemas,
        migrations_path,
        &timestamp,
    );
    build_extend_plan(&mut plan, &extend_planned, migrations_path, &timestamp);
    plan.files.extend(upgrades);

    // ── One atomic commit: dirs → backups → write → lib.rs → admin positioning,
    //    with a single rollback covering all of it.
    let module_count = plan.lib_modules.len();
    commit_plan(&plan, migrations_path)?;

    println!("{}", tf("makemigrations.files_ready", &[module_count]));
    println!("{}", t("makemigrations.apply_hint"));
    warn_unmigrated_type_changes(&destructive_set);

    Ok(())
}
