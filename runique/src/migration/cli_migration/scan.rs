//! Scanning the entities directory for `model!{}` and `extend!{}` declarations.
use crate::migration::*;
use anyhow::{Context, Result};
use std::fs;

/// Tables created by `EihwazUsersMigration` + `AdminTableMigration`, never
/// migrated from the app's entities.
const FRAMEWORK_TABLES: &[&str] = &[
    "eihwaz_users",
    "eihwaz_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_users_groupes",
    "eihwaz_sessions",
    "eihwaz_reset_tokens",
];

/// Scans every `.rs` file in `entities_path` for `model!{}` schemas and
/// parses each one. A `belongs_to` target is resolved the way the `model!{}`
/// macro resolves it — the entity module of that name, i.e. the file
/// `<target>.rs`, or a framework table — and its FK then references that
/// model's real table and primary key; a target that matches nothing is an
/// error. Skips the framework's own tables (`eihwaz_*`).
pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut schemas = Vec::new();
    // List fields' tables: their key to the owner is already resolved.
    let mut list_schemas: Vec<ParsedSchema> = Vec::new();
    // module name → (table, primary key column)
    let mut targets: std::collections::HashMap<String, (String, String)> = FRAMEWORK_TABLES
        .iter()
        .map(|t| (t.to_string(), (t.to_string(), "id".to_string())))
        .collect();
    let mut entries: Vec<_> = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {}", entities_path))?
        .collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Some(module) = path.file_stem().and_then(|n| n.to_str()) else {
            continue;
        };
        if module == "mod" {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        if let Some(ParsedModel { schema, lists, .. }) = parse_model_from_source(&source)
            .with_context(|| format!("Invalid model!{{}} in {}", path.display()))?
        {
            let pk = schema
                .primary_key
                .as_ref()
                .map_or_else(|| "id".to_string(), |pk| pk.name.clone());
            targets.insert(module.to_string(), (schema.table_name.clone(), pk));
            // Ignore tables provided by the framework (`EihwazUsersMigration` + `AdminTableMigration`)
            if FRAMEWORK_TABLES.contains(&schema.table_name.as_str()) {
                continue;
            }
            schemas.push(schema);
            list_schemas.extend(lists);
        }
    }

    for schema in &mut schemas {
        for fk in &mut schema.foreign_keys {
            let Some((table, pk)) = targets.get(&fk.to_table) else {
                anyhow::bail!(
                    "{}: `belongs_to: {} via {}` — no model `{}` ({}/{}.rs) and no framework table of that name",
                    schema.table_name,
                    fk.to_table,
                    fk.from_column,
                    fk.to_table,
                    entities_path,
                    fk.to_table
                );
            };
            fk.to_table = table.clone();
            fk.to_column = pk.clone();
        }
    }

    schemas.extend(list_schemas);
    let mut seen = std::collections::HashSet::new();
    for schema in &schemas {
        if !seen.insert(schema.table_name.as_str()) {
            anyhow::bail!(
                "two tables are named `{}` — a list field's table is `<owner table>_<field>`: rename the field or the other table",
                schema.table_name
            );
        }
    }

    Ok(schemas)
}

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

        let blocks = parse_extend_blocks_from_source(&source)
            .with_context(|| format!("Invalid extend!{{}} in {}", path.display()))?;
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
