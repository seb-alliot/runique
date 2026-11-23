//! The migration plan: computing every change, then writing it in one atomic commit.
use super::lib_file::{lib_modules, planned_lib, strip_sea_orm_cli_placeholder_file};
use super::scan::{merge_extend_schemas, scan_extend_blocks};
use super::snapshot::previous_snapshot;
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::migration::*;
use crate::utils::trad::tf;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// A fully-computed migration plan: everything to create/write/register, no side effects.
/// Built before any IO so the destructive guard and a single atomic commit can run on it.
#[derive(Default)]
pub(super) struct Plan {
    /// (path, content) files to write
    pub(super) files: Vec<(String, String)>,
    /// migration modules to register in `lib.rs`, in order
    pub(super) lib_modules: Vec<String>,
    /// waiting migrations this run replaces: their file is removed and their
    /// module taken out of `lib.rs`
    pub(super) dropped_modules: Vec<String>,
}

/// The migrations of `lib.rs` no `migration up` applied yet (still tagged):
/// their module and their content.
pub(super) fn waiting_migrations(migrations_path: &str) -> Result<Vec<(String, String)>> {
    let mut waiting = Vec::new();
    for module in lib_modules(migrations_path)? {
        if let Ok(content) = fs::read_to_string(format!("{migrations_path}/{module}.rs"))
            && has_marker(&content, WAIT_MIGRATION)
        {
            waiting.push((module, content));
        }
    }
    Ok(waiting)
}

/// Makes `plan`, which holds the migrations just computed and nothing else,
/// take the place of the `waiting` ones. When both say the same thing, table
/// by table, nothing is rewritten: the plan is emptied and `false` returned.
/// Otherwise every waiting migration is dropped, except one the plan writes
/// again under the same name.
pub(super) fn replace_waiting(plan: &mut Plan, waiting: &[(String, String)]) -> bool {
    // The same table and the same content: a creation and an alteration
    // never share one, so the content tells the kind too.
    fn same(waiting: &(String, String), module: &str, content: &str) -> bool {
        migration_target(module) == migration_target(&waiting.0) && content == waiting.1
    }
    let unchanged = waiting.len() == plan.lib_modules.len()
        && waiting.iter().all(|w| {
            plan.lib_modules
                .iter()
                .zip(&plan.files)
                .any(|(m, (_, c))| same(w, m, c))
        })
        && plan
            .lib_modules
            .iter()
            .zip(&plan.files)
            .all(|(m, (_, c))| waiting.iter().any(|w| same(w, m, c)));
    if unchanged {
        plan.files.clear();
        plan.lib_modules.clear();
        return false;
    }
    plan.dropped_modules.extend(
        waiting
            .iter()
            .map(|(module, _)| module)
            .filter(|module| !plan.lib_modules.contains(module))
            .cloned(),
    );
    true
}

/// The migrations of `lib.rs` that are not waiting, in order: what a new
/// migration can start from.
pub(super) fn settled_migrations(
    migrations_path: &str,
    waiting: &[(String, String)],
) -> Result<Vec<String>> {
    Ok(lib_modules(migrations_path)?
        .into_iter()
        .filter(|module| !waiting.iter().any(|(w, _)| w == module))
        .collect())
}

/// Refuses two migrations of `lib.rs` that say they start from the same one
/// for the same table: they were written in parallel, and Runique can't tell
/// which one the table should follow. A migration that doesn't say where it
/// starts from is left out.
pub(super) fn check_parallel(migrations_path: &str) -> Result<()> {
    // (extend, table, started from, module)
    let mut seen: Vec<(bool, String, String, String)> = Vec::new();
    let mut clashes: Vec<String> = Vec::new();
    for module in lib_modules(migrations_path)? {
        let Some((extend, table)) = migration_target(&module) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(format!("{migrations_path}/{module}.rs")) else {
            continue;
        };
        let Some(from) = started_from(&content) else {
            continue;
        };
        if let Some((.., other)) = seen
            .iter()
            .find(|(e, t, f, _)| *e == extend && t == table && f == from)
        {
            clashes.push(format!("{table}: {other}, {module}"));
        }
        seen.push((extend, table.to_string(), from.to_string(), module.clone()));
    }
    if clashes.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "{}",
        tf("makemigrations.parallel_changes", &[clashes.join("\n  ")])
    )
}

/// A generated migration as it is written: tagged as waiting, and saying
/// which migration of `settled` it starts from for its table.
fn waiting_migrate(content: String, module: &str, settled: &[String]) -> String {
    let target = migration_target(module);
    let from = settled
        .iter()
        .rev()
        .find(|m| migration_target(m) == target)
        .map_or("none", String::as_str);
    format!("{WAIT_MIGRATION}\n{FROM_MIGRATION}{from}\n{content}")
}

/// The changes that create the table of `schema` from nothing.
fn creation_of(schema: &ParsedSchema) -> Changes {
    Changes {
        table_name: schema.table_name.clone(),
        added_columns: db_columns(schema).into_iter().cloned().collect(),
        dropped_columns: vec![],
        modified_columns: vec![],
        renames: Default::default(),
        renamed_columns: vec![],
        added_fks: schema.foreign_keys.clone(),
        dropped_fks: vec![],
        added_indexes: schema.indexes.clone(),
        dropped_indexes: vec![],
        is_new_table: true,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    }
}

/// The tables to drop: the ones Runique migrated (`snapshots`, the snapshots
/// of `snapshots/` itself) that no scanned model produces any more, by name.
/// A table of the framework is never dropped, nor one of `kept`: the paused
/// tables, and the former name of a table this run renames.
pub(super) fn dropped_tables(
    snapshots: Vec<ParsedSchema>,
    schemas: &[ParsedSchema],
    kept: &[String],
) -> Vec<ParsedSchema> {
    let mut dropped: Vec<ParsedSchema> = snapshots
        .into_iter()
        .filter(|snapshot| {
            !snapshot.table_name.starts_with(FRAMEWORK_TABLE_PREFIX)
                && !kept.contains(&snapshot.table_name)
                && !schemas.iter().any(|s| s.table_name == snapshot.table_name)
        })
        .collect();
    dropped.sort_by(|a, b| a.table_name.cmp(&b.table_name));
    dropped
}

/// Every table of the framework starts with this.
const FRAMEWORK_TABLE_PREFIX: &str = "eihwaz_";

/// Adds one migration per table to drop, after everything else of the plan:
/// the tables that reference go before the ones they reference, and in a
/// cycle of keys the key still pointing to a table is removed before it.
/// The snapshot stays: `migration up` removes it once the drop is applied.
pub(super) fn build_drop_plan(
    plan: &mut Plan,
    dropped: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
    settled: &[String],
) {
    // Reversed creation order; the keys of a cycle are worked out on the
    // creation order, which is the order the `down`s run in.
    let creation: Vec<Changes> =
        topological_sort_changes(dropped.iter().map(creation_of).collect());
    let cycle_keys = cycle_keys(&creation, dropped);
    let no_cycle = CycleKeys::default();
    for change in drop_order(creation.clone()) {
        let Some(schema) = dropped.iter().find(|s| s.table_name == change.table_name) else {
            continue;
        };
        let keys = cycle_keys
            .get(schema.table_name.as_str())
            .unwrap_or(&no_cycle);
        let module_name = seaorm_drop_module_name(timestamp, &schema.table_name);
        plan.files.push((
            seaorm_drop_file_path(migrations_path, timestamp, &schema.table_name),
            waiting_migrate(generate_drop_file(schema, keys), &module_name, settled),
        ));
        plan.lib_modules.push(module_name);
    }
}

/// The declared renames (`declared`: former table, table) that hold: the
/// former name has a snapshot and no model any more, and the new name has no
/// snapshot yet. Same guard as a column's `renamed_from`: once applied, or
/// when it never matched, the declaration does nothing. Two tables can't
/// both come from one.
pub(super) fn effective_renames(
    declared: &[(String, String)],
    schemas: &[ParsedSchema],
    paused: &[String],
    migrations_path: &str,
) -> Result<Vec<(String, String)>> {
    let has_snapshot =
        |table: &str| Path::new(&snapshot_file_path(migrations_path, table)).exists();
    let mut renames: Vec<(String, String)> = Vec::new();
    for (old, new) in declared {
        if !has_snapshot(old)
            || has_snapshot(new)
            || paused.contains(old)
            || schemas.iter().any(|s| s.table_name == *old)
        {
            continue;
        }
        if let Some((_, other)) = renames.iter().find(|(o, _)| o == old) {
            anyhow::bail!("{}", tf("makemigrations.renamed_twice", &[old, other, new]));
        }
        renames.push((old.clone(), new.clone()));
    }
    Ok(renames)
}

/// Computes the diff for every scanned model (no writing). A table of
/// `renames` (former name, name) is compared to the snapshot of its former
/// name; what the renames explain is taken out of every diff and kept in
/// `Changes::renames`.
pub(super) fn compute_main_changes(
    schemas: &[ParsedSchema],
    renames: &[(String, String)],
    migrations_path: &str,
) -> Result<Vec<Changes>> {
    let mut all_changes: Vec<Changes> = Vec::new();
    for schema in schemas {
        let snapshot_of = renames
            .iter()
            .find(|(_, new)| *new == schema.table_name)
            .map_or(&schema.table_name, |(old, _)| old);
        let snap_path = snapshot_file_path(migrations_path, snapshot_of);
        let changes = if Path::new(&snap_path).exists() {
            let mut previous = previous_snapshot(&snap_path, schema)?;
            let followed = follow_renames(&mut previous, schema, renames);
            Changes {
                renames: followed,
                ..diff_schemas(&previous, schema)
            }
        } else {
            creation_of(schema)
        };
        if !changes.is_empty() {
            all_changes.push(changes);
        }
    }
    Ok(all_changes)
}

/// For each new table, the foreign keys of an FK cycle: keys to a new table
/// created later in this batch (`forward`), and keys pointing to it from a
/// table created earlier (`closing`). `all_changes` is in creation order.
pub(super) fn cycle_keys<'a>(
    all_changes: &[Changes],
    schemas: &'a [ParsedSchema],
) -> std::collections::HashMap<&'a str, CycleKeys<'a>> {
    use std::collections::{HashMap, HashSet};

    let new_tables: HashSet<&str> = all_changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.as_str())
        .collect();
    let mut created: HashSet<&str> = HashSet::new();
    let mut keys: HashMap<&'a str, CycleKeys<'a>> = HashMap::new();
    for change in all_changes.iter().filter(|c| c.is_new_table) {
        let Some(schema) = schemas.iter().find(|s| s.table_name == change.table_name) else {
            continue;
        };
        let table = schema.table_name.as_str();
        for fk in &schema.foreign_keys {
            let target = fk.to_table.as_str();
            if target != table && new_tables.contains(target) && !created.contains(target) {
                keys.entry(table).or_default().forward.push(fk);
                if let Some(target) = schemas.iter().find(|s| s.table_name == target) {
                    keys.entry(target.table_name.as_str())
                        .or_default()
                        .closing
                        .push((table, fk));
                }
            }
        }
        created.insert(table);
    }
    keys
}

/// Adds the main-model migration files/modules to the plan. No snapshot:
/// `migration up` writes it once the migration is applied.
pub(super) fn build_main_plan(
    plan: &mut Plan,
    all_changes: &[Changes],
    schemas: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
    settled: &[String],
) {
    let cycle_keys = cycle_keys(all_changes, schemas);
    let no_cycle = CycleKeys::default();
    for change in all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        if change.is_new_table {
            let module_name = seaorm_create_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, timestamp, &change.table_name);
            let keys = cycle_keys
                .get(change.table_name.as_str())
                .unwrap_or(&no_cycle);
            plan.files.push((
                seaorm_path,
                waiting_migrate(
                    generate_create_file_in_cycle(schema, keys),
                    &module_name,
                    settled,
                ),
            ));
            plan.lib_modules.push(module_name);
        } else {
            let module_name = seaorm_alter_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, timestamp, &change.table_name);
            // What `migration up` reads to take the former snapshot away.
            let renamed = change
                .renames
                .from
                .as_ref()
                .map_or_else(String::new, |old| format!("{RENAMED_TABLE}{old}\n"));
            plan.files.push((
                seaorm_path,
                waiting_migrate(
                    format!("{renamed}{}", generate_alter_file(change)),
                    &module_name,
                    settled,
                ),
            ));
            plan.lib_modules.push(module_name);
        }
    }
}

/// Adds the extend!{} migration files/modules to the plan. No snapshot:
/// `migration up` writes it once the migration is applied.
pub(super) fn build_extend_plan(
    plan: &mut Plan,
    planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
    timestamp: &str,
    settled: &[String],
) {
    if planned.is_empty() {
        return;
    }

    for (ext_schema, changes) in planned {
        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        plan.files.push((
            seaorm_path,
            waiting_migrate(generate_alter_file(changes), &module_name, settled),
        ));
        plan.lib_modules.push(module_name);
    }
}

/// Writes the whole plan as one batch — generated files, replaced migrations
/// removed, and the `lib.rs` that registers them: all of it, or the project
/// as it was.
pub(super) fn commit_plan(plan: &Plan, migrations_path: &str) -> Result<()> {
    let mut batch = AtomicBuilder::modify().journal(migrations_path);
    for (path, content) in &plan.files {
        batch = batch.add(
            PathBuf::from(path),
            SourceItems::Bytes(content.clone().into_bytes()),
            false,
        )?;
    }
    for module in &plan.dropped_modules {
        batch = batch.add(
            PathBuf::from(format!("{migrations_path}/{module}.rs")),
            SourceItems::Remove,
            false,
        )?;
    }
    if let Some(lib) = planned_lib(migrations_path, &plan.lib_modules, &plan.dropped_modules)? {
        batch = batch.add(
            PathBuf::from(lib_path(migrations_path)),
            SourceItems::Bytes(lib.into_bytes()),
            false,
        )?;
    }
    batch.start()?;
    // Runs whether `lib.rs` pre-existed or not: `sea-orm-cli migrate init` always
    // leaves this placeholder behind, and the `lib.rs` just written no longer names it.
    strip_sea_orm_cli_placeholder_file(migrations_path);
    Ok(())
}

/// Scans + merges `extend!{}` blocks and computes their diffs (no writing).
/// Returns the owned schema + change pair for each table that actually changed.
pub(super) fn plan_extend_changes(
    entities_path: &str,
    migrations_path: &str,
) -> Result<Vec<(ParsedSchema, Changes)>> {
    let raw_extends = scan_extend_blocks(entities_path)?;
    if raw_extends.is_empty() {
        return Ok(Vec::new());
    }

    let extend_schemas = merge_extend_schemas(raw_extends);

    let mut planned: Vec<(ParsedSchema, Changes)> = Vec::new();
    for ext_schema in extend_schemas {
        let snap_path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);

        let changes = if Path::new(&snap_path).exists() {
            let previous = previous_snapshot(&snap_path, &ext_schema)?;
            diff_schemas(&previous, &ext_schema)
        } else {
            // First time — all columns are new (ADD COLUMN)
            Changes {
                table_name: ext_schema.table_name.clone(),
                added_columns: ext_schema.columns.clone(),
                dropped_columns: vec![],
                modified_columns: vec![],
                renames: Default::default(),
                renamed_columns: vec![],
                added_fks: vec![],
                dropped_fks: vec![],
                added_indexes: vec![],
                dropped_indexes: vec![],
                is_new_table: false, // always false — the framework table already exists
                enum_renames: vec![],
                enum_value_adds: vec![],
                enum_value_drops: vec![],
            }
        };

        if changes.is_empty() {
            continue;
        }
        planned.push((ext_schema, changes));
    }

    Ok(planned)
}
