use super::*;

/// Scans every `.rs` file in `entities_path` for `model!{}` schemas and
/// parses each one. A `belongs_to` target is resolved the way the `model!{}`
/// macro resolves it — the entity module of that name, i.e. the file
/// `<target>.rs`, or a framework table — and its FK then references that
/// model's real table and primary key; a target that matches nothing is an
/// error. Skips the framework's own tables (`eihwaz_*`).
pub fn scan_models(entities_path: &str) -> Result<ScannedEntities> {
    let mut paused: Vec<String> = Vec::new();
    let mut renames: Vec<(String, String)> = Vec::new();
    let mut schemas = Vec::new();
    let mut enums: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    // List fields' tables: their key to the owner is already resolved.
    let mut list_schemas: Vec<ParsedSchema> = Vec::new();

    // module name → (table, primary key column)
    let mut targets: std::collections::HashMap<String, (String, String)> = FRAMEWORK_TABLES
        .iter()
        .map(|t| (t.to_string(), (t.to_string(), "id".to_string())))
        .collect();

    for path in rust_files(Path::new(entities_path))? {
        let Some(module) = path.file_stem().and_then(|n| n.to_str()) else {
            continue;
        };
        if module == "mod" {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        if let Some(ParsedModel {
            schema,
            lists,
            managed,
            renames: declared,
            ..
        }) = parse_model_from_source(&source)
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
            if !managed {
                paused.push(schema.table_name);
                paused.extend(lists.into_iter().map(|list| list.table_name));
                continue;
            }
            for name in enums_of(&schema, &lists) {
                enums
                    .entry(name)
                    .or_default()
                    .push(schema.table_name.clone());
            }
            schemas.push(schema);
            list_schemas.extend(lists);
            renames.extend(declared);
        }
    }

    for block in scan_extend_blocks(entities_path)? {
        for name in enums_of(&block, &[]) {
            enums
                .entry(name)
                .or_default()
                .push(format!("extend!{{}} {}", block.table_name));
        }
    }

    // One enum name is one type in the database: a second declaration would
    // silently get the values of the first, whose type already exists.
    let declared_twice: Vec<String> = enums
        .into_iter()
        .filter(|(_, models)| models.len() > 1)
        .map(|(name, mut models)| {
            models.sort();
            format!("  {name}: {}", models.join(", "))
        })
        .collect();

    if !declared_twice.is_empty() {
        anyhow::bail!(
            "{}",
            tf(
                runique_i18n::keys::makemigrations::error::ENUM_DECLARED_TWICE,
                &[declared_twice.join("\n")]
            )
        );
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
    for table in schemas.iter().map(|s| &s.table_name).chain(&paused) {
        if !seen.insert(table.as_str()) {
            anyhow::bail!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::error::DUPLICATE_TABLE,
                    &[table]
                )
            );
        }
    }
    Ok(ScannedEntities {
        schemas,
        paused,
        renames,
    })
}
