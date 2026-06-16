//! `makemigration` command — generates SeaORM migration files from `ModelSchema` DSL.
use crate::migration::*;
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

pub use crate::utils::*;

// ── public parse entry points ────────────────────────────────────────────────

pub fn parse_create_file(path: &str) -> Result<ParsedSchema> {
    let source: String =
        fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path))?;
    parse_seaorm_source(&source).with_context(|| format!("Cannot parse: {}", path))
}

// ── scan ─────────────────────────────────────────────────────────────────────

/// Tables created by `EihwazUsersMigration` + `AdminTableMigration`.
/// Excluded from scan when `RUNIQUE_USER_TABLE` is not defined (default user table).
const FRAMEWORK_TABLES: &[&str] = &[
    "eihwaz_users",
    "eihwaz_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_users_groupes",
    "eihwaz_sessions",
    "eihwaz_reset_tokens",
];

pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    dotenvy::dotenv().ok();
    let using_builtin_user = std::env::var("RUNIQUE_USER_TABLE")
        .unwrap_or_default()
        .is_empty()
        || std::env::var("RUNIQUE_USER_TABLE").unwrap_or_default() == "eihwaz_users";

    let mut schemas = Vec::new();
    let mut model_table_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let entries = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {}", entities_path))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        if let Some((model_name, schema)) = parse_schema_from_source(&source) {
            // Ignore tables provided by the framework (`EihwazUsersMigration` + `AdminTableMigration`)
            if using_builtin_user && FRAMEWORK_TABLES.contains(&schema.table_name.as_str()) {
                continue;
            }
            if !model_name.is_empty() {
                model_table_map.insert(model_name, schema.table_name.clone());
            }
            schemas.push(schema);
        }
    }

    // Fix FK to_table: pascal_to_snake(ModelName) → actual table_name from meta
    for schema in &mut schemas {
        for fk in &mut schema.foreign_keys {
            if let Some(table) = model_table_map.get(&fk.to_table) {
                fk.to_table = table.clone();
            }
        }
    }

    Ok(schemas)
}

// ── lib.rs updater ───────────────────────────────────────────────────────────

/// `module_name` must be in SeaORM format: `m{timestamp}_create_{table}_table`
pub fn update_migration_lib(migrations_path: &str, module_name: &str) -> Result<()> {
    let lib = lib_path(migrations_path);
    let mod_line = format!("mod {};", module_name);
    let box_line = format!("            Box::new({}::Migration),", module_name);

    if !Path::new(&lib).exists() {
        let content = format!(
            "use runique::prelude::migrations_table;\nuse sea_orm_migration::prelude::*;\n\n{}\n\npub struct Migrator;\n\n#[async_trait::async_trait]\nimpl MigratorTrait for Migrator {{\n    fn migrations() -> Vec<Box<dyn MigrationTrait>> {{\n        vec![\n{}\n        ]\n    }}\n}}\n",
            mod_line, box_line
        );
        fs::write(&lib, content)?;
    } else {
        let existing = fs::read_to_string(&lib)?;
        if existing.contains(&mod_line) {
            return Ok(());
        }
        // Add `use migrations_table` if missing (`lib.rs` existing without it)
        let existing = if !existing.contains("migrations_table") {
            existing.replacen(
                "use sea_orm_migration::prelude::*;",
                "use runique::prelude::migrations_table;\nuse sea_orm_migration::prelude::*;",
                1,
            )
        } else {
            existing
        };
        let updated = existing
            .replacen(
                "use sea_orm_migration::prelude::*;",
                &format!("use sea_orm_migration::prelude::*;\n{}", mod_line),
                1,
            )
            .replacen("        ]", &format!("{}\n        ]", box_line), 1);
        fs::write(&lib, updated)?;
    }

    Ok(())
}

// ── db kind detection ────────────────────────────────────────────────────────

/// Detects the DB backend from `DB_URL` or `DB_ENGINE` in `.env`.
/// Used to generate DB-specific SQL (trigger vs ON UPDATE).
fn detect_db_kind() -> crate::migration::utils::types::DbKind {
    dotenvy::dotenv().ok();
    use crate::migration::utils::types::DbKind;

    let url = std::env::var("DB_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_default();

    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        return DbKind::Postgres;
    } else if url.starts_with("mysql://") || url.starts_with("mariadb://") {
        return DbKind::Mysql;
    }

    match std::env::var("DB_ENGINE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "postgres" | "postgresql" => DbKind::Postgres,
        "mysql" | "mariadb" => DbKind::Mysql,
        _ => DbKind::Other,
    }
}

// ── topological sort ─────────────────────────────────────────────────────────

/// Sorts `Changes` by FK dependency order.
/// Tables referenced by other new tables are placed first.
/// Existing tables (not new) are ignored: they already exist in DB.
/// In case of circular dependency, the remaining tables are added at the end.
pub(crate) fn topological_sort_changes(
    changes: Vec<crate::migration::utils::types::Changes>,
) -> Vec<crate::migration::utils::types::Changes> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let new_tables: HashSet<String> = changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.clone())
        .collect();

    // deps[A] = {B} : A has a FK to B, so B must be created before A
    let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
    for change in &changes {
        if !change.is_new_table {
            continue;
        }
        let entry = deps.entry(change.table_name.clone()).or_default();
        for fk in &change.added_fks {
            if new_tables.contains(&fk.to_table) && fk.to_table != change.table_name {
                entry.insert(fk.to_table.clone());
            }
        }
    }

    // dependents[B] = [A] : when B is processed, we can decrement the in_degree of A
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for (table, table_deps) in &deps {
        for dep in table_deps {
            dependents
                .entry(dep.clone())
                .or_default()
                .push(table.clone());
        }
    }

    // in_degree[A] = number of requirements for A (tables A waits for via FK)
    // Tables without requirements (in_degree == 0) are processed first.
    let mut in_degree: HashMap<String, usize> = new_tables.iter().map(|t| (t.clone(), 0)).collect();
    for (table, table_deps) in &deps {
        let deg = in_degree.entry(table.clone()).or_insert(0);
        *deg = deg.saturating_add(table_deps.len());
    }

    // Kahn's algorithm: starts with tables without requirements (e.g. B referenced by A)
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(t, _)| t.clone())
        .collect();
    let mut sorted_names: Vec<String> = Vec::new();

    while let Some(table) = queue.pop_front() {
        sorted_names.push(table.clone());
        if let Some(dependents_list) = dependents.get(&table) {
            for dep in dependents_list {
                let entry = in_degree.entry(dep.clone()).or_insert(1);
                if *entry > 0 {
                    *entry = entry.saturating_sub(1);
                }
                if *entry == 0 {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    let mut result: Vec<crate::migration::utils::types::Changes> =
        Vec::with_capacity(changes.len());
    let mut by_name: HashMap<String, crate::migration::utils::types::Changes> = changes
        .into_iter()
        .map(|c| (c.table_name.clone(), c))
        .collect();

    // New tables in topological order (referenced first)
    for name in sorted_names {
        if let Some(c) = by_name.remove(&name) {
            result.push(c);
        }
    }
    // Remaining (ALTER + potential circles)
    result.extend(by_name.into_values());
    result
}

// ── destructive change guard ─────────────────────────────────────────────────

pub fn collect_destructive_messages(all_changes: &[Changes]) -> Vec<String> {
    let dropped = all_changes.iter().flat_map(|c| {
        c.dropped_columns
            .iter()
            .map(|col| format!("  {}.{}: DROP COLUMN (data loss)", c.table_name, col.name))
    });

    let type_changes = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| old.col_type != new.col_type)
            .map(|(old, new)| {
                format!(
                    "  {}.{}: type {} -> {}",
                    c.table_name, old.name, old.col_type, new.col_type
                )
            })
    });

    let nullable_to_required = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| old.nullable && !new.nullable && old.col_type == new.col_type)
            .map(|(_, new)| {
                format!(
                    "  {}.{}: nullable -> not_null (requires a default or backfill)",
                    c.table_name, new.name
                )
            })
    });

    let dropped_fks = all_changes.iter().flat_map(|c| {
        c.dropped_fks.iter().map(|fk| {
            format!(
                "  {}.{}: DROP FOREIGN KEY -> {} (orphan records possible)",
                c.table_name, fk.from_column, fk.to_table
            )
        })
    });

    // Adding a CASCADE constraint to existing data can trigger mass deletes if a parent is removed.
    // Only a concern on existing tables: a table created in this same batch has no rows yet,
    // so a CASCADE FK on a brand-new table carries no data-loss risk (avoids a false positive
    // on first generation / from-scratch regeneration).
    let cascade_fks = all_changes
        .iter()
        .filter(|c| !c.is_new_table)
        .flat_map(|c| {
            c.added_fks
                .iter()
                .filter(|fk| fk.on_delete.to_uppercase() == "CASCADE")
                .map(|fk| {
                    format!(
                        "  {}.{}: ADD FOREIGN KEY -> {} ON DELETE CASCADE (existing rows may be deleted)",
                        c.table_name, fk.from_column, fk.to_table
                    )
                })
        });

    dropped
        .chain(type_changes)
        .chain(nullable_to_required)
        .chain(dropped_fks)
        .chain(cascade_fks)
        .collect()
}

fn check_destructive(all_changes: &[Changes], force: bool) -> Result<()> {
    let blocking = collect_destructive_messages(all_changes);

    if blocking.is_empty() || force {
        return Ok(());
    }

    eprintln!("\n{}", t("makemigrations.destructive_detected"));
    for msg in &blocking {
        eprintln!("{}", msg);
    }
    anyhow::bail!("{}", t("makemigrations.destructive_require_force"));
}

// ── run ──────────────────────────────────────────────────────────────────────
pub fn seaorm_alter_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_alter_{}_table", timestamp, table)
}

pub fn seaorm_alter_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/{}.rs",
        migrations_path,
        seaorm_alter_module_name(timestamp, table)
    )
}

pub fn seaorm_extend_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_extend_{}_table", timestamp, table)
}

pub fn seaorm_extend_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/m{}_extend_{}_table.rs",
        migrations_path, timestamp, table
    )
}

// ── scan extend blocks ────────────────────────────────────────────────────────

/// Scans all `.rs` files in the `entities_path` directory and collects
/// all found `extend!{}` blocks. Returns a flat list of `ParsedSchema`
/// (one per block — multiple blocks can target the same table).
pub fn scan_extend_blocks(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut schemas = Vec::new();
    let entries = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {}", entities_path))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        let blocks = parse_extend_blocks_from_source(&source);
        for schema in blocks {
            schemas.push(schema);
        }
    }
    Ok(schemas)
}

/// Merges `extend!{}` blocks targeting the same table into a single `ParsedSchema`.
/// Columns are concatenated in discovery order.
pub fn merge_extend_schemas(schemas: Vec<ParsedSchema>) -> Vec<ParsedSchema> {
    use std::collections::HashMap;
    let mut by_table: HashMap<String, Vec<ParsedColumn>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for schema in schemas {
        if !by_table.contains_key(&schema.table_name) {
            order.push(schema.table_name.clone());
            by_table.insert(schema.table_name.clone(), Vec::new());
        }
        by_table
            .get_mut(&schema.table_name)
            .unwrap()
            .extend(schema.columns);
    }

    order
        .into_iter()
        .map(|table_name| {
            let columns = by_table.remove(&table_name).unwrap_or_default();
            ParsedSchema {
                table_name,
                primary_key: None,
                columns,
                foreign_keys: Vec::new(),
                indexes: Vec::new(),
            }
        })
        .collect()
}
pub fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    let schemas = scan_entities(entities_path)?;

    fs::create_dir_all(applied_dir(migrations_path))?;
    fs::create_dir_all(snapshot_dir(migrations_path))?;

    // ── Plan everything up front — nothing is written until the full plan
    //    (main models + extend!{} blocks) is computed and validated.
    let mut main_changes = compute_main_changes(&schemas, migrations_path)?;
    let extend_planned = plan_extend_changes(entities_path, migrations_path)?;

    if main_changes.is_empty() && extend_planned.is_empty() {
        return Ok(());
    }

    // ── Single destructive guard over main + extend (honors --force) ──────
    let mut destructive_set: Vec<Changes> = main_changes.clone();
    destructive_set.extend(extend_planned.iter().map(|(_, c)| c.clone()));
    check_destructive(&destructive_set, force)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let db_kind = detect_db_kind();

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
        &db_kind,
    );
    build_extend_plan(
        &mut plan,
        &extend_planned,
        migrations_path,
        &timestamp,
        &db_kind,
    );

    // ── One atomic commit: dirs → backups → write → lib.rs → admin positioning,
    //    with a single rollback covering all of it.
    let module_count = plan.lib_modules.len();
    commit_plan(&plan, migrations_path)?;

    println!("{}", tf("makemigrations.files_ready", &[module_count]));

    Ok(())
}

// ── unified plan ───────────────────────────────────────────────────────────────

/// A fully-computed migration plan: everything to create/write/register, no side effects.
/// Built before any IO so the destructive guard and a single atomic commit can run on it.
#[derive(Default)]
struct Plan {
    /// (path, content) files to write
    files: Vec<(String, String)>,
    /// directories to create before writing
    dirs: Vec<String>,
    /// migration modules to register in `lib.rs`, in order
    lib_modules: Vec<String>,
}

/// Computes the diff for every scanned model (no writing).
fn compute_main_changes(schemas: &[ParsedSchema], migrations_path: &str) -> Result<Vec<Changes>> {
    let mut all_changes: Vec<Changes> = Vec::new();
    for schema in schemas {
        let snap_path = snapshot_file_path(migrations_path, &schema.table_name);
        let changes = if Path::new(&snap_path).exists() {
            let previous = parse_create_file(&snap_path)?;
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

/// Adds the main-model migration files/dirs/modules to the plan.
fn build_main_plan(
    plan: &mut Plan,
    all_changes: &[Changes],
    schemas: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
    db_kind: &crate::migration::utils::types::DbKind,
) {
    // Schemas of new tables that have FK constraints — collected for the relations migration.
    let mut schemas_with_fks: Vec<&ParsedSchema> = Vec::new();

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
            // CREATE TABLE without FK constraints — relations are added last via relations migration.
            plan.files
                .push((seaorm_path, generate_create_file(schema, db_kind)));
            plan.lib_modules.push(module_name);
            if !schema.foreign_keys.is_empty() {
                schemas_with_fks.push(schema);
            }
        } else {
            plan.dirs
                .push(table_applied_dir(migrations_path, &change.table_name));
            plan.dirs
                .push(batch_up_dir(migrations_path, &change.table_name));
            plan.dirs
                .push(batch_down_dir(migrations_path, &change.table_name));

            let module_name = seaorm_alter_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, timestamp, &change.table_name);
            plan.files
                .push((seaorm_path, generate_alter_file(change, db_kind)));
            plan.lib_modules.push(module_name);

            plan.files.push((
                alter_file_path(migrations_path, &change.table_name, timestamp),
                generate_alter_file(change, db_kind),
            ));
            plan.files.push((
                batch_up_path(migrations_path, &change.table_name, timestamp),
                generate_batch_up_file(&[change], timestamp),
            ));
            plan.files.push((
                batch_down_path(migrations_path, &change.table_name, timestamp),
                generate_batch_down_file(&[change], timestamp),
            ));
        }
    }

    // Relations migration — all FK constraints grouped in a single migration, registered
    // after every CREATE so the referenced tables already exist when constraints are added.
    // SQLite is excluded: it cannot ALTER-ADD FKs, so they are inlined in each CREATE TABLE.
    let inline_fk_engine = *db_kind == crate::migration::utils::types::DbKind::Other;
    if !schemas_with_fks.is_empty() && !inline_fk_engine {
        let module_name = format!("m{}_create_relations", timestamp);
        let path = format!("{}/{}.rs", migrations_path, module_name);
        plan.files
            .push((path, generate_relations_file(&schemas_with_fks)));
        plan.lib_modules.push(module_name);
    }
}

/// Adds the extend!{} migration files/dirs/modules to the plan.
fn build_extend_plan(
    plan: &mut Plan,
    planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
    timestamp: &str,
    db_kind: &crate::migration::utils::types::DbKind,
) {
    if planned.is_empty() {
        return;
    }
    plan.dirs.push(extend_snapshot_dir(migrations_path));

    for (ext_schema, changes) in planned {
        // Snapshot updated (without PK, just extension columns)
        plan.files.push((
            extend_snapshot_file_path(migrations_path, &ext_schema.table_name),
            generate_create_file(ext_schema, &crate::migration::utils::types::DbKind::Other),
        ));

        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        plan.files
            .push((seaorm_path, generate_alter_file(changes, db_kind)));
        plan.lib_modules.push(module_name);

        plan.dirs
            .push(table_applied_dir(migrations_path, &ext_schema.table_name));
        plan.dirs
            .push(batch_up_dir(migrations_path, &ext_schema.table_name));
        plan.dirs
            .push(batch_down_dir(migrations_path, &ext_schema.table_name));

        plan.files.push((
            alter_file_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_alter_file(changes, db_kind),
        ));
        plan.files.push((
            batch_up_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_batch_up_file(&[changes], timestamp),
        ));
        plan.files.push((
            batch_down_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_batch_down_file(&[changes], timestamp),
        ));
    }
}

/// Writes the whole plan atomically: create dirs, back up existing targets, write files,
/// register lib.rs modules, position AdminTableMigration — all under a single rollback.
fn commit_plan(plan: &Plan, migrations_path: &str) -> Result<()> {
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
    let mut file_backups: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
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

// ── AdminTableMigration positioning ───────────────────────────────────────

/// Automatically positions `AdminTableMigration` in `lib.rs` just after the migration
/// of the user table (`RUNIQUE_USER_TABLE`, default: `eihwaz_users`).
///
/// - Adds `use runique::prelude::migrations_table;` if missing
/// - Removes `AdminTableMigration` from its current position if present
/// - Inserts it immediately after the line `Box::new(<user_table_migration>)`
/// - No effect if the user table migration is not yet in `lib.rs`
pub fn ensure_admin_migration_positioned(migrations_path: &str) -> Result<()> {
    dotenvy::dotenv().ok();
    let user_table = crate::admin::table_admin::migrations_table::user_table_name();
    let lib_file = lib_path(migrations_path);

    if !Path::new(&lib_file).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_file)?;

    // Ensure `use runique::prelude::migrations_table;` is present
    let content = if !content.contains("migrations_table") {
        content.replacen(
            "use sea_orm_migration::prelude::*;",
            "use runique::prelude::migrations_table;\nuse sea_orm_migration::prelude::*;",
            1,
        )
    } else {
        content
    };

    let admin_box = "            Box::new(migrations_table::AdminTableMigration),";
    let sessions_box = "            Box::new(migrations_table::EihwazSessionsMigration),";
    let reset_box = "            Box::new(migrations_table::EihwazResetTokensMigration),";
    let users_box = "            Box::new(migrations_table::EihwazUsersMigration),";
    let user_pattern = format!("create_{}_table", user_table);

    let using_builtin_user = user_table == "eihwaz_users";

    // Tables created by `EihwazUsersMigration` + `EihwazSessionsMigration` + `AdminTableMigration` — to exclude from the vec
    const FRAMEWORK_TABLE_PATTERNS: &[&str] = &[
        "create_eihwaz_users_table",
        "create_eihwaz_groupes_table",
        "create_eihwaz_groupes_droits_table",
        "create_eihwaz_users_groupes_table",
        "create_eihwaz_sessions_table",
        "create_eihwaz_reset_tokens_table",
    ];

    if using_builtin_user {
        // ── Default case: user table provided by the framework ──────────────
        // Remove existing framework lines (we'll re-inject them at the top)
        // and also filter app migrations duplicating framework tables.
        let mut lines: Vec<String> = content
            .lines()
            .filter(|l| {
                !l.contains("migrations_table::EihwazUsersMigration")
                    && !l.contains("migrations_table::EihwazSessionsMigration")
                    && !l.contains("migrations_table::EihwazResetTokensMigration")
                    && !l.contains("migrations_table::AdminTableMigration")
                    && !FRAMEWORK_TABLE_PATTERNS.iter().any(|pat| l.contains(pat))
            })
            .map(|l| l.to_string())
            .collect();

        // Insert all three framework migrations at the start of vec![
        if let Some(idx) = lines
            .iter()
            .position(|l| l.trim() == "vec![" || l.contains("vec!["))
        {
            lines.insert(idx + 1, reset_box.to_string());
            lines.insert(idx + 1, admin_box.to_string());
            lines.insert(idx + 1, sessions_box.to_string());
            lines.insert(idx + 1, users_box.to_string());
        }

        let result = lines.join("\n") + "\n";
        if result != content {
            fs::write(&lib_file, &result)?;
        }
    } else {
        // ── Custom case: the dev provides their own user table ──────────────────
        // AdminTableMigration positioned right after the migration of its table
        if !content.contains(&user_pattern) {
            fs::write(&lib_file, &content)?;
            return Ok(());
        }

        let mut lines: Vec<String> = content
            .lines()
            .filter(|l| {
                !l.contains("migrations_table::AdminTableMigration")
                    && !l.contains("migrations_table::EihwazResetTokensMigration")
            })
            .map(|l| l.to_string())
            .collect();

        if let Some(idx) = lines.iter().position(|l| l.contains(&user_pattern)) {
            lines.insert(idx + 1, reset_box.to_string());
            lines.insert(idx + 1, admin_box.to_string());
        }

        let result = lines.join("\n") + "\n";
        if result != content {
            fs::write(&lib_file, &result)?;
        }
    }

    Ok(())
}

// ── Extend pass (planning) ─────────────────────────────────────────────────────

/// Scans + merges `extend!{}` blocks and computes their diffs (no writing).
/// Returns the owned schema + change pair for each table that actually changed.
fn plan_extend_changes(
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
            let previous = parse_create_file(&snap_path)?;
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
