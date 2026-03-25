use crate::migration::*;
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::{
    io::{self, Write},
    path::Path,
};

pub use crate::utils::*;

// ── public parse entry points ────────────────────────────────────────────────

pub fn parse_create_file(path: &str) -> Result<ParsedSchema> {
    let source: String =
        fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path))?;
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
            println!(
                "  {}",
                tf("makemigrations.found_schema", &[&schema.table_name])
            );
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

    println!(" {}", tf("makemigrations.updated_lib", &[&lib]));
    Ok(())
}

// ── db kind detection ────────────────────────────────────────────────────────

/// Détecte le backend DB depuis `DB_URL` ou `DB_ENGINE` dans le `.env`.
/// Utilisé pour générer le SQL DB-spécifique (trigger vs ON UPDATE).
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

/// Trie les `Changes` par ordre de dépendance FK.
/// Les tables référencées par d'autres nouvelles tables sont placées en premier.
/// Les tables existantes (non nouvelles) sont ignorées : elles existent déjà en DB.
/// En cas de dépendance circulaire, les tables restantes sont ajoutées à la fin.
fn topological_sort_changes(
    changes: Vec<crate::migration::utils::types::Changes>,
) -> Vec<crate::migration::utils::types::Changes> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let new_tables: HashSet<String> = changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.clone())
        .collect();

    // Dépendances : table → tables dont elle dépend (parmi les nouvelles tables)
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

    // Algorithme de Kahn
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

    for (table, table_deps) in &deps {
        in_degree.entry(table.clone()).or_insert(0);
        for dep in table_deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 0; // assure l'entrée
            dependents
                .entry(dep.clone())
                .or_default()
                .push(table.clone());
        }
        for dep in table_deps {
            *in_degree.get_mut(dep).unwrap_or(&mut 0) += 0;
        }
    }

    // in_degree correct : compter combien de tables dépendent de chacune
    let mut in_degree: HashMap<String, usize> = new_tables.iter().map(|t| (t.clone(), 0)).collect();
    for table_deps in deps.values() {
        for dep in table_deps {
            *in_degree.entry(dep.clone()).or_insert(0) += 1;
        }
    }

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
                    *entry -= 1;
                }
                if *entry == 0 {
                    queue.push_back(dep.clone());
                }
            }
        }
    }

    // Tables non nouvelles : conserver leur ordre original à la fin
    let mut result: Vec<crate::migration::utils::types::Changes> =
        Vec::with_capacity(changes.len());
    let mut by_name: HashMap<String, crate::migration::utils::types::Changes> = changes
        .into_iter()
        .map(|c| (c.table_name.clone(), c))
        .collect();

    // Nouvelles tables triées topologiquement
    for name in sorted_names {
        if let Some(c) = by_name.remove(&name) {
            result.push(c);
        }
    }
    // Restants (alter + éventuels cycles)
    result.extend(by_name.into_values());
    result
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
pub async fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    println!(" {}", tf("makemigrations.scanning", &[entities_path]));

    let schemas = scan_entities(entities_path)?;
    if schemas.is_empty() {
        println!(" {}", tf("makemigrations.no_schema", &[entities_path]));
        return Ok(());
    }
    println!(
        " {}",
        tf("makemigrations.schema_count", &[&schemas.len().to_string()])
    );

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
        println!(" {}", t("makemigrations.no_changes"));
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
        println!("\n{}", t("makemigrations.destructive_detected"));
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
    let db_kind = detect_db_kind();

    // ── tri topologique des nouvelles tables ─────────────────────────────────
    // Les tables référencées par FK doivent être créées AVANT les tables qui
    // les référencent, car sea-orm exécute les migrations dans l'ordre de lib.rs.
    all_changes = topological_sort_changes(all_changes);

    // ── write files ──────────────────────────────────────────────────────────
    for change in &all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        // Snapshot always updated (DbKind::Other — pas de SQL DB-spécifique dans le diff)
        let snap_path = snapshot_file_path(migrations_path, &change.table_name);
        fs::write(
            &snap_path,
            generate_create_file(schema, &crate::migration::utils::types::DbKind::Other),
        )
        .with_context(|| format!("Failed to write snapshot: {}", snap_path))?;

        if change.is_new_table {
            // Timestamped SeaORM file — immutable, executed by sea-orm-cli
            let module_name = seaorm_create_module_name(&timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, &timestamp, &change.table_name);

            fs::write(&seaorm_path, generate_create_file(schema, &db_kind))
                .with_context(|| format!("Failed to write: {}", seaorm_path))?;

            println!(" {}", tf("makemigrations.generated", &[&seaorm_path]));
            update_migration_lib(migrations_path, &module_name)?;
        } else {
            println!(" {}", tf("makemigrations.snapshot_updated", &[&snap_path]));

            // ALTER file in applied/<table>/
            let table_dir = table_applied_dir(migrations_path, &change.table_name);
            fs::create_dir_all(&table_dir)?;

            let alter_path = alter_file_path(migrations_path, &change.table_name, &timestamp);
            if !change.is_new_table {
                let module_name = seaorm_alter_module_name(&timestamp, &change.table_name);
                let seaorm_path =
                    seaorm_alter_file_path(migrations_path, &timestamp, &change.table_name);

                fs::write(&seaorm_path, generate_alter_file(change)).with_context(|| {
                    format!("Failed to write SeaORM alter migration: {}", seaorm_path)
                })?;

                println!("{}", tf("makemigrations.seaorm_alter", &[&seaorm_path]));

                update_migration_lib(migrations_path, &module_name)?;
            }
            fs::write(&alter_path, generate_alter_file(change))
                .with_context(|| format!("Failed to write: {}", alter_path))?;
            println!(" {}", tf("makemigrations.generated", &[&alter_path]));

            // Batch up/down per table in applied/by_time/<table>/
            let up_dir = batch_up_dir(migrations_path, &change.table_name);
            let down_dir = batch_down_dir(migrations_path, &change.table_name);
            fs::create_dir_all(&up_dir)?;
            fs::create_dir_all(&down_dir)?;

            let up_path = batch_up_path(migrations_path, &change.table_name, &timestamp);
            fs::write(&up_path, generate_batch_up_file(&[change], &timestamp))
                .with_context(|| format!("Failed to write batch up: {}", up_path))?;
            println!(" {}", tf("makemigrations.batch_up", &[&up_path]));

            let down_path = batch_down_path(migrations_path, &change.table_name, &timestamp);
            fs::write(&down_path, generate_batch_down_file(&[change], &timestamp))
                .with_context(|| format!("Failed to write batch down: {}", down_path))?;
            println!(" {}", tf("makemigrations.batch_down", &[&down_path]));
        }
    }

    println!("\n{}", t("makemigrations.apply_hint"));
    Ok(())
}
