use crate::migration::*;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub use crate::utils::*;

// ── public parse entry points ────────────────────────────────────────────────

pub fn parse_create_file(path: &str) -> Result<ParsedSchema> {
    let source = fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path))?;
    parse_seaorm_source(&source).with_context(|| format!("Cannot parse: {}", path))
}

// ── scan ─────────────────────────────────────────────────────────────────────

pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
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

        if let Some(schema) = parse_schema_from_source(&source) {
            println!("  -> Found schema: {}", schema.table_name);
            schemas.push(schema);
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
            "use sea_orm_migration::prelude::*;\n\n{}\n\npub struct Migrator;\n\n#[async_trait::async_trait]\nimpl MigratorTrait for Migrator {{\n    fn migrations() -> Vec<Box<dyn MigrationTrait>> {{\n        vec![\n{}\n        ]\n    }}\n}}\n",
            mod_line, box_line
        );
        fs::write(&lib, content)?;
    } else {
        let existing = fs::read_to_string(&lib)?;
        if existing.contains(&mod_line) {
            return Ok(());
        }
        let updated = existing
            .replacen(
                "use sea_orm_migration::prelude::*;",
                &format!("use sea_orm_migration::prelude::*;\n{}", mod_line),
                1,
            )
            .replacen("        ]", &format!("{}\n        ]", box_line), 1);
        fs::write(&lib, updated)?;
    }

    println!(" Updated {}", lib);
    Ok(())
}

// ── run ──────────────────────────────────────────────────────────────────────

pub async fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    println!(" Scanning entities in '{}'...", entities_path);

    let schemas = scan_entities(entities_path)?;
    if schemas.is_empty() {
        println!(" No schema() functions found in '{}'.", entities_path);
        return Ok(());
    }
    println!(" Found {} schema(s).", schemas.len());

    fs::create_dir_all(applied_dir(migrations_path))?;
    fs::create_dir_all(snapshot_dir(migrations_path))?;

    let mut all_changes: Vec<Changes> = Vec::new();

    for schema in &schemas {
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
                added_fks: schema.foreign_keys.clone(),
                dropped_fks: vec![],
                added_indexes: schema.indexes.clone(),
                dropped_indexes: vec![],
                is_new_table: true,
            }
        };
        if !changes.is_empty() {
            all_changes.push(changes);
        }
    }

    if all_changes.is_empty() {
        println!(" No changes detected. Your schema is up to date.");
        return Ok(());
    }

    // ── destructive check ────────────────────────────────────────────────────
    let type_changes: Vec<String> = all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| old.col_type != new.col_type)
                .map(|(old, new)| {
                    format!(
                        "  {}.{}: type {} -> {}",
                        c.table_name, old.name, old.col_type, new.col_type
                    )
                })
        })
        .collect();

    let nullable_to_required: Vec<String> = all_changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| old.nullable && !new.nullable && old.col_type == new.col_type)
                .map(|(_, new)| {
                    format!(
                        "  {}.{}: nullable -> not_null (requires a default or backfill)",
                        c.table_name, new.name
                    )
                })
        })
        .collect();

    let blocking: Vec<String> = [type_changes, nullable_to_required].concat();

    if !blocking.is_empty() && !force {
        println!("\nDestructive changes detected:");
        for msg in &blocking {
            println!("{}", msg);
        }
        print!("\nProvide a default value for migration, or use --force to skip: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().is_empty() {
            anyhow::bail!("Destructive changes require a default value or --force. Aborting.");
        }
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    // ── write files ──────────────────────────────────────────────────────────
    for change in &all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        // Snapshot always updated
        let snap_path = snapshot_file_path(migrations_path, &change.table_name);
        fs::write(&snap_path, generate_create_file(schema))
            .with_context(|| format!("Failed to write snapshot: {}", snap_path))?;

        if change.is_new_table {
            // Timestamped SeaORM file — immutable, executed by sea-orm-cli
            let module_name = seaorm_create_module_name(&timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, &timestamp, &change.table_name);

            fs::write(&seaorm_path, generate_create_file(schema))
                .with_context(|| format!("Failed to write: {}", seaorm_path))?;

            println!(" Generated: {}", seaorm_path);
            update_migration_lib(migrations_path, &module_name)?;
        } else {
            println!(" Updated snapshot: {}", snap_path);

            // ALTER file in applied/<table>/
            let table_dir = table_applied_dir(migrations_path, &change.table_name);
            fs::create_dir_all(&table_dir)?;

            let alter_path = alter_file_path(migrations_path, &change.table_name, &timestamp);
            fs::write(&alter_path, generate_alter_file(change))
                .with_context(|| format!("Failed to write: {}", alter_path))?;
            println!(" Generated: {}", alter_path);

            // Batch up/down per table in applied/by_time/<table>/
            let up_dir = batch_up_dir(migrations_path, &change.table_name);
            let down_dir = batch_down_dir(migrations_path, &change.table_name);
            fs::create_dir_all(&up_dir)?;
            fs::create_dir_all(&down_dir)?;

            let up_path = batch_up_path(migrations_path, &change.table_name, &timestamp);
            fs::write(&up_path, generate_batch_up_file(&[change], &timestamp))
                .with_context(|| format!("Failed to write batch up: {}", up_path))?;
            println!(" Generated batch up:   {}", up_path);

            let down_path = batch_down_path(migrations_path, &change.table_name, &timestamp);
            fs::write(&down_path, generate_batch_down_file(&[change], &timestamp))
                .with_context(|| format!("Failed to write batch down: {}", down_path))?;
            println!(" Generated batch down: {}", down_path);
        }
    }

    println!("\nRun 'sea-orm-cli migrate up' to apply.");
    Ok(())
}
