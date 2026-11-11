//! The migration plan: computing every change, then writing it in one atomic commit.
use super::lib_file::{ensure_admin_migration_positioned, update_migration_lib};
use super::scan::{merge_extend_schemas, scan_extend_blocks};
use super::snapshot::{extend_snapshot_content, previous_snapshot};
use crate::migration::*;
use crate::utils::StrMap;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// A fully-computed migration plan: everything to create/write/register, no side effects.
/// Built before any IO so the destructive guard and a single atomic commit can run on it.
#[derive(Default)]
pub(super) struct Plan {
    /// (path, content) files to write
    pub(super) files: Vec<(String, String)>,
    /// directories to create before writing
    pub(super) dirs: Vec<String>,
    /// migration modules to register in `lib.rs`, in order
    pub(super) lib_modules: Vec<String>,
}

/// Computes the diff for every scanned model (no writing).
pub(super) fn compute_main_changes(
    schemas: &[ParsedSchema],
    migrations_path: &str,
) -> Result<Vec<Changes>> {
    let mut all_changes: Vec<Changes> = Vec::new();
    for schema in schemas {
        let snap_path = snapshot_file_path(migrations_path, &schema.table_name);
        let changes = if Path::new(&snap_path).exists() {
            let previous = previous_snapshot(&snap_path, schema)?;
            diff_schemas(&previous, schema)
        } else {
            Changes {
                table_name: schema.table_name.clone(),
                added_columns: db_columns(schema).into_iter().cloned().collect(),
                dropped_columns: vec![],
                modified_columns: vec![],
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

/// Adds the main-model migration files/dirs/modules to the plan.
pub(super) fn build_main_plan(
    plan: &mut Plan,
    all_changes: &[Changes],
    schemas: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
) {
    let cycle_keys = cycle_keys(all_changes, schemas);
    let no_cycle = CycleKeys::default();
    for change in all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        // Snapshot — includes FK stmts so future diffs detect FK additions/removals.
        plan.files.push((
            snapshot_file_path(migrations_path, &change.table_name),
            generate_snapshot_file(schema),
        ));

        if change.is_new_table {
            let module_name = seaorm_create_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, timestamp, &change.table_name);
            let keys = cycle_keys
                .get(change.table_name.as_str())
                .unwrap_or(&no_cycle);
            plan.files
                .push((seaorm_path, generate_create_file_in_cycle(schema, keys)));
            plan.lib_modules.push(module_name);
        } else {
            let module_name = seaorm_alter_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, timestamp, &change.table_name);
            plan.files.push((seaorm_path, generate_alter_file(change)));
            plan.lib_modules.push(module_name);
        }
    }
}

/// Adds the extend!{} migration files/dirs/modules to the plan.
pub(super) fn build_extend_plan(
    plan: &mut Plan,
    planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
    timestamp: &str,
) {
    if planned.is_empty() {
        return;
    }
    plan.dirs.push(extend_snapshot_dir(migrations_path));

    for (ext_schema, changes) in planned {
        // Snapshot updated (without PK, just extension columns)
        plan.files.push((
            extend_snapshot_file_path(migrations_path, &ext_schema.table_name),
            extend_snapshot_content(ext_schema),
        ));

        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        plan.files.push((seaorm_path, generate_alter_file(changes)));
        plan.lib_modules.push(module_name);
    }
}

/// Writes the whole plan atomically: create dirs, back up existing targets, write files,
/// register lib.rs modules, position AdminTableMigration — all under a single rollback.
pub(super) fn commit_plan(plan: &Plan, migrations_path: &str) -> Result<()> {
    // Directory creation (idempotent)
    for dir in &plan.dirs {
        fs::create_dir_all(dir)?;
    }

    // lib.rs backup for rollback in case of partial error.
    let lib_file = lib_path(migrations_path);
    let lib_backup: Option<String> = if Path::new(&lib_file).exists() {
        Some(fs::read_to_string(&lib_file)?)
    } else {
        None
    };

    // Back up the previous content of any target that already exists (snapshots especially),
    // so the rollback restores it instead of deleting it — a deleted snapshot would make the
    // next run regenerate a full CREATE for an already-migrated table.
    let mut file_backups: StrMap = StrMap::new();
    for (path, _) in &plan.files {
        if Path::new(path).exists()
            && let Ok(prev) = fs::read_to_string(path)
        {
            file_backups.insert(path.clone(), prev);
        }
    }

    let mut written: Vec<String> = Vec::new();
    let write_result: Result<()> = (|| {
        for (path, content) in &plan.files {
            fs::write(path, content).with_context(|| format!("Failed to write: {}", path))?;
            written.push(path.clone());
        }
        for module_name in &plan.lib_modules {
            update_migration_lib(migrations_path, module_name)?;
        }
        // AdminTableMigration positioning rewrites lib.rs — kept inside the protected
        // scope so a failure here also triggers the rollback below.
        ensure_admin_migration_positioned(migrations_path)?;
        Ok(())
    })();

    if let Err(e) = write_result {
        eprintln!(
            "\n[makemigrations] Error: {}. Rollback generated files...",
            e
        );
        for path in &written {
            match file_backups.get(path) {
                Some(prev) => {
                    if let Err(re) = fs::write(path, prev) {
                        eprintln!("  warning: cannot restore {} : {}", path, re);
                    } else {
                        eprintln!("  restored: {}", path);
                    }
                }
                None => {
                    if let Err(re) = fs::remove_file(path) {
                        eprintln!("  warning: cannot delete {} : {}", path, re);
                    } else {
                        eprintln!("  deleted: {}", path);
                    }
                }
            }
        }
        match lib_backup {
            Some(content) => {
                let _ = fs::write(&lib_file, content);
                eprintln!("  lib.rs restored");
            }
            None => {
                let _ = fs::remove_file(&lib_file);
            }
        }
        return Err(e);
    }

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
