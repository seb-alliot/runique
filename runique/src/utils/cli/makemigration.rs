//! Commande `makemigration` — génère les fichiers de migration SeaORM à partir du DSL `ModelSchema`.
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

/// Tables créées par EihwazUsersMigration + AdminTableMigration.
/// Exclues du scan quand `RUNIQUE_USER_TABLE` n'est pas défini (table user par défaut).
const FRAMEWORK_TABLES: &[&str] = &[
    "eihwaz_users",
    "eihwaz_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_users_groupes",
];

pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    dotenvy::dotenv().ok();
    let using_builtin_user = std::env::var("RUNIQUE_USER_TABLE")
        .unwrap_or_default()
        .is_empty()
        || std::env::var("RUNIQUE_USER_TABLE").unwrap_or_default() == "eihwaz_users";

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
            // Ignore les tables fournies par le framework (EihwazUsersMigration + AdminTableMigration)
            if using_builtin_user && FRAMEWORK_TABLES.contains(&schema.table_name.as_str()) {
                continue;
            }
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
            "use runique::prelude::migrations_table;\nuse sea_orm_migration::prelude::*;\n\n{}\n\npub struct Migrator;\n\n#[async_trait::async_trait]\nimpl MigratorTrait for Migrator {{\n    fn migrations() -> Vec<Box<dyn MigrationTrait>> {{\n        vec![\n{}\n        ]\n    }}\n}}\n",
            mod_line, box_line
        );
        fs::write(&lib, content)?;
    } else {
        let existing = fs::read_to_string(&lib)?;
        if existing.contains(&mod_line) {
            return Ok(());
        }
        // Ajoute le `use migrations_table` si absent (lib.rs existant sans lui)
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

    // deps[A] = {B} : A a une FK vers B, donc B doit être créée avant A
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

    // dependents[B] = [A] : quand B est traité, on peut décrémenter l'in_degree de A
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for (table, table_deps) in &deps {
        for dep in table_deps {
            dependents
                .entry(dep.clone())
                .or_default()
                .push(table.clone());
        }
    }

    // in_degree[A] = nombre de prérequis de A (tables que A attend via FK)
    // Les tables sans prérequis (in_degree == 0) sont traitées en premier.
    let mut in_degree: HashMap<String, usize> = new_tables.iter().map(|t| (t.clone(), 0)).collect();
    for (table, table_deps) in &deps {
        let deg = in_degree.entry(table.clone()).or_insert(0);
        *deg = deg.saturating_add(table_deps.len());
    }

    // Algorithme de Kahn : commence par les tables sans prérequis (ex. B référencée par A)
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

    // Nouvelles tables dans l'ordre topologique (référencées en premier)
    for name in sorted_names {
        if let Some(c) = by_name.remove(&name) {
            result.push(c);
        }
    }
    // Restants (ALTER + éventuels cycles)
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

/// Scanne tous les fichiers `.rs` du répertoire `entities_path` et collecte
/// tous les blocs `extend!{}` trouvés. Retourne une liste plate de `ParsedSchema`
/// (un par bloc — plusieurs blocs peuvent cibler la même table).
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

/// Fusionne les blocs `extend!{}` qui ciblent la même table en un seul `ParsedSchema`.
/// Les colonnes sont concaténées dans l'ordre de découverte.
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
    if schemas.is_empty() {
        return Ok(());
    }

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
                enum_renames: vec![],
                enum_value_adds: vec![],
                enum_value_drops: vec![],
            }
        };
        if !changes.is_empty() {
            all_changes.push(changes);
        }
    }

    if all_changes.is_empty() {
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

    // ── tri topologique : tables référencées créées avant celles qui les référencent
    all_changes = topological_sort_changes(all_changes);

    // ── Phase 1 : planification — aucune écriture ────────────────────────────
    let mut files_to_write: Vec<(String, String)> = Vec::new(); // (path, content)
    let mut extra_dirs: Vec<String> = Vec::new();
    let mut lib_modules: Vec<String> = Vec::new();

    for change in &all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        // Snapshot (toujours mis à jour, sans SQL DB-spécifique)
        files_to_write.push((
            snapshot_file_path(migrations_path, &change.table_name),
            generate_create_file(schema, &crate::migration::utils::types::DbKind::Other),
        ));

        if change.is_new_table {
            let module_name = seaorm_create_module_name(&timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, &timestamp, &change.table_name);
            files_to_write.push((seaorm_path, generate_create_file(schema, &db_kind)));
            lib_modules.push(module_name);
        } else {
            extra_dirs.push(table_applied_dir(migrations_path, &change.table_name));
            extra_dirs.push(batch_up_dir(migrations_path, &change.table_name));
            extra_dirs.push(batch_down_dir(migrations_path, &change.table_name));

            let module_name = seaorm_alter_module_name(&timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, &timestamp, &change.table_name);
            files_to_write.push((seaorm_path, generate_alter_file(change)));
            lib_modules.push(module_name);

            files_to_write.push((
                alter_file_path(migrations_path, &change.table_name, &timestamp),
                generate_alter_file(change),
            ));
            files_to_write.push((
                batch_up_path(migrations_path, &change.table_name, &timestamp),
                generate_batch_up_file(&[change], &timestamp),
            ));
            files_to_write.push((
                batch_down_path(migrations_path, &change.table_name, &timestamp),
                generate_batch_down_file(&[change], &timestamp),
            ));
        }
    }

    // ── Phase 2 : création des répertoires (idempotent) ──────────────────────
    for dir in &extra_dirs {
        fs::create_dir_all(dir)?;
    }

    // ── Phase 3 : écriture atomique ──────────────────────────────────────────
    // Sauvegarde de lib.rs pour rollback en cas d'erreur partielle.
    let lib_file = lib_path(migrations_path);
    let lib_backup: Option<String> = if Path::new(&lib_file).exists() {
        Some(fs::read_to_string(&lib_file)?)
    } else {
        None
    };

    let mut written: Vec<String> = Vec::new();

    let write_result: Result<()> = (|| {
        for (path, content) in &files_to_write {
            fs::write(path, content).with_context(|| format!("Failed to write: {}", path))?;
            written.push(path.clone());
        }
        for module_name in &lib_modules {
            update_migration_lib(migrations_path, module_name)?;
        }
        Ok(())
    })();

    if let Err(e) = write_result {
        eprintln!(
            "\n[makemigrations] Erreur : {}. Annulation des fichiers générés...",
            e
        );
        for path in &written {
            if let Err(re) = fs::remove_file(path) {
                eprintln!(
                    "  avertissement : impossible de supprimer {} : {}",
                    path, re
                );
            } else {
                eprintln!("  supprimé : {}", path);
            }
        }
        match lib_backup {
            Some(content) => {
                let _ = fs::write(&lib_file, content);
                eprintln!("  lib.rs restauré");
            }
            None => {
                let _ = fs::remove_file(&lib_file);
            }
        }
        return Err(e);
    }

    // ── Passe 2 : extensions de tables framework (extend!{}) ─────────────────
    run_extend_pass(entities_path, migrations_path, &timestamp, &db_kind)?;

    // ── Passe 3 : positionnement automatique d'AdminTableMigration ────────────
    ensure_admin_migration_positioned(migrations_path)?;

    println!("{}", tf("makemigrations.files_ready", &[lib_modules.len()]));

    Ok(())
}

// ── Positionnement AdminTableMigration ───────────────────────────────────────

/// Positionne automatiquement `AdminTableMigration` dans `lib.rs` juste après la migration
/// de la table user (`RUNIQUE_USER_TABLE`, défaut : `eihwaz_users`).
///
/// - Ajoute `use runique::prelude::migrations_table;` si absent
/// - Retire `AdminTableMigration` de sa position actuelle si présente
/// - L'insère immédiatement après la ligne `Box::new(<user_table_migration>)`
/// - Sans effet si la migration de la user table n'est pas encore dans `lib.rs`
pub fn ensure_admin_migration_positioned(migrations_path: &str) -> Result<()> {
    dotenvy::dotenv().ok();
    let user_table = crate::admin::table_admin::migrations_table::user_table_name();
    let lib_file = lib_path(migrations_path);

    if !Path::new(&lib_file).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_file)?;

    // Ensure `use runique::prelude::migrations_table;` est présent
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
    let users_box = "            Box::new(migrations_table::EihwazUsersMigration),";
    let user_pattern = format!("create_{}_table", user_table);

    let using_builtin_user = user_table == "eihwaz_users";

    // Tables créées par EihwazUsersMigration + AdminTableMigration — à exclure du vec
    const FRAMEWORK_TABLE_PATTERNS: &[&str] = &[
        "create_eihwaz_users_table",
        "create_eihwaz_groupes_table",
        "create_eihwaz_groupes_droits_table",
        "create_eihwaz_users_groupes_table",
    ];

    if using_builtin_user {
        // ── Cas par défaut : table user fournie par le framework ──────────────
        // Retire les lignes framework existantes (on va les réinjecter en tête)
        // et filtre aussi les migrations app qui dupliquent les tables framework.
        let mut lines: Vec<String> = content
            .lines()
            .filter(|l| {
                !l.contains("migrations_table::EihwazUsersMigration")
                    && !l.contains("migrations_table::AdminTableMigration")
                    && !FRAMEWORK_TABLE_PATTERNS.iter().any(|pat| l.contains(pat))
            })
            .map(|l| l.to_string())
            .collect();

        // Insère les deux migrations framework au début du vec![
        if let Some(idx) = lines
            .iter()
            .position(|l| l.trim() == "vec![" || l.contains("vec!["))
        {
            lines.insert(idx + 1, admin_box.to_string());
            lines.insert(idx + 1, users_box.to_string());
        }

        let result = lines.join("\n") + "\n";
        if result != content {
            fs::write(&lib_file, &result)?;
        }
    } else {
        // ── Cas custom : le dev fournit sa propre table user ──────────────────
        // AdminTableMigration positionnée juste après la migration de sa table
        if !content.contains(&user_pattern) {
            fs::write(&lib_file, &content)?;
            return Ok(());
        }

        let mut lines: Vec<String> = content
            .lines()
            .filter(|l| !l.contains("migrations_table::AdminTableMigration"))
            .map(|l| l.to_string())
            .collect();

        if let Some(idx) = lines.iter().position(|l| l.contains(&user_pattern)) {
            lines.insert(idx + 1, admin_box.to_string());
        }

        let result = lines.join("\n") + "\n";
        if result != content {
            fs::write(&lib_file, &result)?;
        }
    }

    Ok(())
}

// ── Passe extend ──────────────────────────────────────────────────────────────

fn run_extend_pass(
    entities_path: &str,
    migrations_path: &str,
    timestamp: &str,
    _db_kind: &crate::migration::utils::types::DbKind,
) -> Result<()> {
    let raw_extends = scan_extend_blocks(entities_path)?;
    if raw_extends.is_empty() {
        return Ok(());
    }

    let extend_schemas = merge_extend_schemas(raw_extends);
    fs::create_dir_all(extend_snapshot_dir(migrations_path))?;

    let mut files_to_write: Vec<(String, String)> = Vec::new();
    let mut lib_modules: Vec<String> = Vec::new();

    for ext_schema in &extend_schemas {
        let snap_path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);

        let changes = if Path::new(&snap_path).exists() {
            let previous = parse_create_file(&snap_path)?;
            diff_schemas(&previous, ext_schema)
        } else {
            // Première fois — toutes les colonnes sont nouvelles (ADD COLUMN)
            Changes {
                table_name: ext_schema.table_name.clone(),
                added_columns: ext_schema.columns.clone(),
                dropped_columns: vec![],
                modified_columns: vec![],
                added_fks: vec![],
                dropped_fks: vec![],
                added_indexes: vec![],
                dropped_indexes: vec![],
                is_new_table: false, // toujours false — la table framework existe déjà
                enum_renames: vec![],
                enum_value_adds: vec![],
                enum_value_drops: vec![],
            }
        };

        if changes.is_empty() {
            continue;
        }

        // Snapshot mis à jour (sans PK, juste les colonnes d'extension)
        files_to_write.push((
            snap_path,
            generate_create_file(ext_schema, &crate::migration::utils::types::DbKind::Other),
        ));

        // Fichier de migration SeaORM extend
        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        files_to_write.push((seaorm_path, generate_alter_file(&changes)));
        lib_modules.push(module_name);

        // Fichiers applied/batch (répertoire propre pour chaque table étendue)
        let apply_dir = table_applied_dir(migrations_path, &ext_schema.table_name);
        let up_dir = batch_up_dir(migrations_path, &ext_schema.table_name);
        let down_dir = batch_down_dir(migrations_path, &ext_schema.table_name);
        fs::create_dir_all(&apply_dir)?;
        fs::create_dir_all(&up_dir)?;
        fs::create_dir_all(&down_dir)?;

        files_to_write.push((
            alter_file_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_alter_file(&changes),
        ));
        files_to_write.push((
            batch_up_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_batch_up_file(&[&changes], timestamp),
        ));
        files_to_write.push((
            batch_down_path(migrations_path, &ext_schema.table_name, timestamp),
            generate_batch_down_file(&[&changes], timestamp),
        ));
    }

    if files_to_write.is_empty() {
        return Ok(());
    }

    // Écriture
    let lib_backup: Option<String> = {
        let lib_file = lib_path(migrations_path);
        if Path::new(&lib_file).exists() {
            Some(fs::read_to_string(&lib_file)?)
        } else {
            None
        }
    };
    let lib_file = lib_path(migrations_path);
    let mut written: Vec<String> = Vec::new();

    let write_result: Result<()> = (|| {
        for (path, content) in &files_to_write {
            fs::write(path, content).with_context(|| format!("Failed to write: {}", path))?;
            written.push(path.clone());
        }
        for module_name in &lib_modules {
            update_migration_lib(migrations_path, module_name)?;
        }
        Ok(())
    })();

    if let Err(e) = write_result {
        eprintln!("\n[makemigrations extend] Erreur : {}. Annulation...", e);
        for path in &written {
            if let Err(re) = fs::remove_file(path) {
                eprintln!(
                    "  avertissement : impossible de supprimer {} : {}",
                    path, re
                );
            }
        }
        if let Some(content) = lib_backup {
            let _ = fs::write(&lib_file, content);
        }
        return Err(e);
    }

    Ok(())
}
