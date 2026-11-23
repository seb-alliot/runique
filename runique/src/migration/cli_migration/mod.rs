//! Migration commands' logic: `makemigrations` (connects, then `run` generates SeaORM migration
//! files from the `model!{}` DSL), `migration up/down/status` (`migrate`) and the
//! project's `migration` crate (`init_migrate`). The commands themselves are
//! called through `cli::makemigration` and `cli::migrate`.
mod applied;
mod applied_renames;
pub(crate) mod init_migrate;
mod lib_file;
pub mod migrate;
mod plan;
mod report;
mod scan;
mod snapshot;
#[cfg(test)]
mod tests;

use crate::db::DatabaseConfig;
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
pub use scan::{
    ScannedEntities, merge_extend_schemas, scan_entities, scan_extend_blocks, scan_models,
};
pub use snapshot::parse_create_file;
// For the integration tests that replay the reading on PostgreSQL and MySQL.
pub use applied::read as applied_migrations;
// Only the pipeline tests (`migration/tests_pipeline.rs`) reach it from outside.
#[cfg(test)]
pub(crate) use snapshot::previous_snapshot;

use plan::{
    Plan, build_drop_plan, build_extend_plan, build_main_plan, check_parallel, commit_plan,
    compute_main_changes, dropped_tables, effective_renames, plan_extend_changes, replace_waiting,
    settled_migrations, waiting_migrations,
};
use report::{check_destructive, check_identifier_lengths, warn_unmigrated_type_changes};
use snapshot::snapshot_upgrades;

/// Entry point of the `makemigrations` command: scans models and `extend!{}`
/// blocks, diffs them against their last snapshot, blocks on destructive
/// changes unless `force` is set, then writes the full migration plan atomically.
///
/// A snapshot is the state of a table once `migration up` applied it: this
/// command never writes one. Run again before anything was applied, it starts
/// from the same snapshots and its result takes the place of the migrations
/// still waiting, instead of piling on them.
pub fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    let ScannedEntities {
        schemas,
        paused,
        renames,
    } = scan_models(entities_path)?;
    if !paused.is_empty() {
        println!(
            "{}",
            tf("makemigrations.paused_tables", &[paused.join(", ")])
        );
    }
    init_migrate::ensure(migrations_path)?;
    // Before anything reads the snapshots: a run the power cut short is taken
    // back, so the changes are computed on migrations that are whole.
    init_migrate::repair_beside(
        std::path::Path::new(migrations_path),
        std::path::Path::new(entities_path),
    )?;

    let migrated = snapshot::read_all(migrations_path)?;
    check_parallel(migrations_path)?;
    let waiting = waiting_migrations(migrations_path)?;

    fs::create_dir_all(snapshot_dir(migrations_path))?;

    // ── Plan everything up front — nothing is written until the full plan
    //    (main models + extend!{} blocks) is computed and validated.
    let renames = effective_renames(&renames, &schemas, &paused, migrations_path)?;
    let mut main_changes = compute_main_changes(&schemas, &renames, migrations_path)?;
    let extend_planned = plan_extend_changes(entities_path, migrations_path)?;
    // A table Runique migrated that no model produces any more. A paused
    // table stays, and so does the former name of a renamed one.
    let kept: Vec<String> = paused
        .iter()
        .chain(renames.iter().map(|(old, _)| old))
        .cloned()
        .collect();
    let dropped = dropped_tables(migrated, &schemas, &kept);

    let upgrades = snapshot_upgrades(
        entities_path,
        &schemas,
        &main_changes,
        &extend_planned,
        migrations_path,
    )?;

    if main_changes.is_empty() && extend_planned.is_empty() && dropped.is_empty() {
        // A project without a model of its own still needs the framework's
        // tables (accounts, sessions): wired into `lib.rs` even then, and the
        // `sea-orm-cli migrate init` placeholder dropped.
        let upgraded = upgrades.len();
        // Nothing left to migrate: what was waiting no longer matches the
        // models (a change taken back before it was applied).
        let plan = Plan {
            files: upgrades,
            dropped_modules: waiting.iter().map(|(module, _)| module.clone()).collect(),
            ..Plan::default()
        };
        commit_plan(&plan, migrations_path)?;
        if upgraded != 0 {
            println!("{}", tf("makemigrations.snapshots_upgraded", &[upgraded]));
        }
        if !waiting.is_empty() {
            println!("{}", tf("makemigrations.waiting_removed", &[waiting.len()]));
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
    let dropped_names: Vec<String> = dropped.iter().map(|s| s.table_name.clone()).collect();
    check_destructive(&destructive_set, &dropped_names, force)?;
    check_identifier_lengths(&destructive_set)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    // referenced tables created before those referencing them
    main_changes = topological_sort_changes(main_changes);
    settle_renamed_keys(&mut main_changes)?;

    // ── Build a single unified plan (no writing) ──────────────────────────
    let settled = settled_migrations(migrations_path, &waiting)?;
    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &main_changes,
        &schemas,
        migrations_path,
        &timestamp,
        &settled,
    );
    build_extend_plan(
        &mut plan,
        &extend_planned,
        migrations_path,
        &timestamp,
        &settled,
    );
    // Last: what destroys closes the run, once the keys of the tables that
    // stay were removed by their own migrations above.
    build_drop_plan(&mut plan, &dropped, migrations_path, &timestamp, &settled);
    // While the plan holds the migrations and nothing else.
    let rewritten = replace_waiting(&mut plan, &waiting);
    plan.files.extend(upgrades);

    // ── One atomic commit: dirs → backups → write → lib.rs → admin positioning,
    //    with a single rollback covering all of it.
    let module_count = plan.lib_modules.len();
    commit_plan(&plan, migrations_path)?;

    if rewritten {
        println!("{}", tf("makemigrations.files_ready", &[module_count]));
    } else {
        println!(
            "{}",
            tf("makemigrations.waiting_unchanged", &[waiting.len()])
        );
    }
    println!("{}", t("makemigrations.apply_hint"));
    warn_unmigrated_type_changes(&destructive_set);

    Ok(())
}

/// `makemigrations`: connects first, then plans and writes. An unreachable
/// database stops the command before anything is read or written.
pub async fn makemigrations(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    dotenvy::dotenv().ok();
    makemigrations_with(
        |key| std::env::var(key).ok(),
        entities_path,
        migrations_path,
        force,
    )
    .await
}

pub(crate) async fn makemigrations_with(
    get: impl Fn(&str) -> Option<String>,
    entities_path: &str,
    migrations_path: &str,
    force: bool,
) -> Result<()> {
    let mut config = DatabaseConfig::from_lookup(get)
        .map_err(anyhow::Error::msg)?
        .build();
    // The command only ever needs one connection.
    config.pool(1, 1);
    let db = config
        .connect()
        .await
        .map_err(|e| anyhow::anyhow!(tf("makemigrations.database_unreachable", &[e])))?;
    // Before `lib.rs` is read: a run the power cut short may have left it
    // half written.
    init_migrate::repair_beside(
        std::path::Path::new(migrations_path),
        std::path::Path::new(entities_path),
    )?;
    let applied = applied::read(&db).await?;
    applied::check_known(&applied, migrations_path)?;
    run(entities_path, migrations_path, force)
}
