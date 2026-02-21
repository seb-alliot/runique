use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend};
use std::fs;
use std::path::Path;

// ============================================================
// Public API
// ============================================================

pub async fn up(migrations_path: &str) -> Result<()> {
    println!("Applying migrations from '{}'...", migrations_path);
    println!("Run: sea-orm-cli migrate up");
    Ok(())
}

pub async fn down(migrations_path: &str, files: Vec<String>, batch: Option<String>) -> Result<()> {
    if files.is_empty() && batch.is_none() {
        list_available(migrations_path)?;
        return Ok(());
    }

    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .with_context(|| "DATABASE_URL not set. Add it to your .env file.")?;

    let db = Database::connect(&db_url)
        .await
        .with_context(|| "Failed to connect to database.")?;

    if let Some(batch_ts) = batch {
        rollback_batch(migrations_path, &batch_ts, &db).await?;
    } else {
        for file_arg in &files {
            rollback_file(migrations_path, file_arg, &db).await?;
        }
    }

    println!("\nRollback complete.");
    Ok(())
}

pub async fn status(migrations_path: &str) -> Result<()> {
    println!("Available rollback files for '{}':", migrations_path);
    list_available(migrations_path)?;
    Ok(())
}

// ============================================================
// Rollback: batch + file
// ============================================================

async fn rollback_batch(
    migrations_path: &str,
    timestamp: &str,
    db: &DatabaseConnection,
) -> Result<()> {
    let by_time_dir = format!("{}/applied/by_time", migrations_path);
    let batch_file = format!("{}/{}.rs", by_time_dir, timestamp);

    if !Path::new(&batch_file).exists() {
        anyhow::bail!(
            "Batch '{}' not found.\nRun 'sea-orm-builder migrate status' to list available batches.",
            timestamp
        );
    }

    check_order_batch(&by_time_dir, timestamp)?;

    println!("Rolling back batch: {}", timestamp);

    let source = fs::read_to_string(&batch_file)
        .with_context(|| format!("Cannot read batch file: {}", batch_file))?;

    execute_down_block(&source, db).await?;

    println!("  Done: {}", timestamp);
    Ok(())
}

async fn rollback_file(
    migrations_path: &str,
    file_arg: &str,
    db: &DatabaseConnection,
) -> Result<()> {
    let applied_dir = format!("{}/applied", migrations_path);

    let file_path = if file_arg.ends_with(".rs") {
        format!("{}/{}", applied_dir, file_arg)
    } else {
        format!("{}/{}.rs", applied_dir, file_arg)
    };

    if !Path::new(&file_path).exists() {
        anyhow::bail!(
            "File not found: {}\nRun 'sea-orm-builder migrate status' to list available files.",
            file_path
        );
    }

    // enforce rollback order per table if file_arg is "table/timestamp"
    let parts: Vec<&str> = file_arg.trim_end_matches(".rs").splitn(2, '/').collect();
    if parts.len() == 2 {
        let table = parts[0];
        let timestamp = parts[1];
        let table_dir = format!("{}/{}", applied_dir, table);
        check_order_file(&table_dir, timestamp)?;
    }

    println!("Rolling back: {}", file_arg);

    let source = fs::read_to_string(&file_path)
        .with_context(|| format!("Cannot read file: {}", file_path))?;

    execute_down_block(&source, db).await?;

    println!("  Done: {}", file_arg);
    Ok(())
}

// ============================================================
// Rollback order checks
// ============================================================

fn check_order_batch(by_time_dir: &str, timestamp: &str) -> Result<()> {
    let mut batches = list_files_in_dir(by_time_dir)?;
    batches.sort();

    if let Some(latest) = batches.last() {
        let latest_ts = latest.trim_end_matches(".rs");
        if latest_ts != timestamp {
            anyhow::bail!(
                "Cannot rollback batch '{}'.\nA more recent batch exists: '{}'.\nRollback most recent first.",
                timestamp,
                latest_ts
            );
        }
    }

    Ok(())
}

fn check_order_file(table_dir: &str, timestamp: &str) -> Result<()> {
    let mut files = list_files_in_dir(table_dir)?;
    files.sort();

    if let Some(latest) = files.last() {
        let latest_ts = latest.trim_end_matches(".rs");
        if latest_ts != timestamp {
            anyhow::bail!(
                "Cannot rollback '{}'.\nA more recent migration exists for this table: '{}'.\nRollback most recent first.",
                timestamp,
                latest_ts
            );
        }
    }

    Ok(())
}

// ============================================================
// Listing
// ============================================================

fn list_files_in_dir(dir: &str) -> Result<Vec<String>> {
    if !Path::new(dir).exists() {
        return Ok(vec![]);
    }

    let files = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    Ok(files)
}

fn list_available(migrations_path: &str) -> Result<()> {
    let applied_dir = format!("{}/applied", migrations_path);

    if !Path::new(&applied_dir).exists() {
        println!("No applied/ directory found.");
        return Ok(());
    }

    let mut found = false;

    let mut tables: Vec<String> = fs::read_dir(&applied_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != "by_time")
        .collect();
    tables.sort();

    for table in &tables {
        let table_dir = format!("{}/{}", applied_dir, table);
        let mut files = list_files_in_dir(&table_dir)?;
        files.sort();

        if !files.is_empty() {
            found = true;
            println!("\n  {}:", table);
            for file in &files {
                println!("    {}/{}", table, file.trim_end_matches(".rs"));
            }
        }
    }

    let by_time_dir = format!("{}/by_time", applied_dir);
    let mut batches = list_files_in_dir(&by_time_dir)?;
    batches.sort();

    if !batches.is_empty() {
        found = true;
        println!("\n  by_time (full batches):");
        for batch in &batches {
            println!("    {}", batch.trim_end_matches(".rs"));
        }
    }

    if !found {
        println!("No rollback files available.");
    } else {
        println!("\nUsage:");
        println!(
            "  sea-orm-builder migrate down --files <table/timestamp> [<table/timestamp> ...]"
        );
        println!("  sea-orm-builder migrate down --batch <timestamp>");
    }

    Ok(())
}

// ============================================================
// Core: execute down()
// ============================================================

async fn execute_down_block(source: &str, db: &DatabaseConnection) -> Result<()> {
    let backend = db.get_database_backend();

    // 1) capture down() block safely
    let down_block = extract_fn_block(source, "down").unwrap_or_default();

    // 2) build SQL statements from down() content
    let statements = extract_statements_from_block(&down_block, source, backend);

    if statements.is_empty() {
        println!("  No down statements found.");
        return Ok(());
    }

    for sql in &statements {
        println!("  Executing: {}", sql);
        db.execute_unprepared(sql)
            .await
            .with_context(|| format!("Failed to execute: {}", sql))?;
    }

    Ok(())
}

// ============================================================
// Extract function block
// ============================================================

fn extract_fn_block(source: &str, fn_name: &str) -> Option<String> {
    let mut in_fn = false;
    let mut depth: i32 = 0;
    let mut out = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if !in_fn && trimmed.contains(&format!("async fn {}", fn_name)) {
            in_fn = true;

            out.push_str(line);
            out.push('\n');

            depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

            if depth == 0 {
                return Some(out);
            }
            continue;
        }

        if !in_fn {
            continue;
        }

        out.push_str(line);
        out.push('\n');

        depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
        depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

        if depth == 0 {
            return Some(out);
        }
    }

    None
}

// ============================================================
// SQL extraction (expanded)
// ============================================================

fn extract_statements_from_block(
    block: &str,
    full_source: &str,
    backend: DbBackend,
) -> Vec<String> {
    let mut statements = Vec::new();

    // table name: try block first, fallback to whole file
    let table_name = extract_table_from_source(block)
        .or_else(|| extract_table_from_source(full_source))
        .unwrap_or_default();

    if table_name.is_empty() {
        return statements;
    }

    for line in block.lines() {
        let trimmed = line.trim();

        // ----------------------------
        // add_column
        // ----------------------------
        if trimmed.contains(".add_column(") && trimmed.contains("Alias::new(\"") {
            if let Some(col) = extract_alias_value(trimmed) {
                let col_type = seaorm_sql_type(trimmed);
                let null = if trimmed.contains(".null()") {
                    ""
                } else {
                    " NOT NULL"
                };

                let sql = match backend {
                    DbBackend::Postgres => format!(
                        "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}{};",
                        table_name, col, col_type, null
                    ),
                    _ => format!(
                        "ALTER TABLE `{}` ADD COLUMN `{}` {}{};",
                        table_name, col, col_type, null
                    ),
                };
                statements.push(sql);
            }
        }

        // ----------------------------
        // drop_column
        // ----------------------------
        if trimmed.contains(".drop_column(Alias::new(\"") {
            if let Some(col) = extract_alias_value(trimmed) {
                let sql = match backend {
                    DbBackend::Postgres => {
                        format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\";", table_name, col)
                    }
                    _ => format!("ALTER TABLE `{}` DROP COLUMN `{}`;", table_name, col),
                };
                statements.push(sql);
            }
        }

        // ----------------------------
        // modify_column : nullable only (SET/DROP NOT NULL)
        // Pattern attendu: .modify_column(ColumnDef::new(Alias::new("bio")).text().not_null())
        // ----------------------------
        if trimmed.contains(".modify_column(") && trimmed.contains("Alias::new(\"") {
            if let Some(col) = extract_alias_value(trimmed) {
                // Ici on ne gère que NOT NULL / NULL (suffisant pour ton test)
                let make_null = trimmed.contains(".null()");
                let sql = match backend {
                    DbBackend::Postgres => {
                        if make_null {
                            format!(
                                "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" DROP NOT NULL;",
                                table_name, col
                            )
                        } else {
                            format!(
                                "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET NOT NULL;",
                                table_name, col
                            )
                        }
                    }
                    _ => {
                        // MySQL/MariaDB : MODIFY COLUMN nécessite le type. On fait un fallback "unsafe".
                        // => Si tu vises MySQL, il faudra passer sur sea_query pour être correct.
                        if make_null {
                            format!("-- WARNING: modify_column NULL not supported safely on this backend for `{}`.`{}`", table_name, col)
                        } else {
                            format!("-- WARNING: modify_column NOT NULL not supported safely on this backend for `{}`.`{}`", table_name, col)
                        }
                    }
                };
                statements.push(sql);
            }
        }

        // ----------------------------
        // drop_table
        // ----------------------------
        if trimmed.contains(".drop_table(") && trimmed.contains("Alias::new(\"") {
            if let Some(t) = extract_alias_value(trimmed) {
                let sql = match backend {
                    DbBackend::Postgres => format!("DROP TABLE IF EXISTS \"{}\";", t),
                    _ => format!("DROP TABLE IF EXISTS `{}`;", t),
                };
                statements.push(sql);
            }
        }

        // ----------------------------
        // drop_index (pattern: Index::drop().name("idx").table(Alias::new("t")))
        // ----------------------------
        if trimmed.contains(".drop_index(") && trimmed.contains(".name(\"") {
            if let Some(idx) = extract_name_value(trimmed) {
                let sql = match backend {
                    DbBackend::Postgres => format!("DROP INDEX IF EXISTS \"{}\";", idx),
                    _ => format!("DROP INDEX `{}`;", idx),
                };
                statements.push(sql);
            }
        }

        // ----------------------------
        // drop_foreign_key (pattern: ForeignKey::drop().table(Alias::new("t")).name("t_col_ref_fkey"))
        // ----------------------------
        if trimmed.contains(".drop_foreign_key(") && trimmed.contains(".name(\"") {
            if let Some(fk_name) = extract_name_value(trimmed) {
                let sql = match backend {
                    DbBackend::Postgres => format!(
                        "ALTER TABLE \"{}\" DROP CONSTRAINT IF EXISTS \"{}\";",
                        table_name, fk_name
                    ),
                    _ => format!(
                        "ALTER TABLE `{}` DROP FOREIGN KEY `{}`;",
                        table_name, fk_name
                    ),
                };
                statements.push(sql);
            }
        }
    }

    statements
}

// ============================================================
// Helpers: extract table + alias + name("...")
// ============================================================

fn extract_table_from_source(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains(".table(Alias::new(\"") {
            return extract_alias_value(trimmed);
        }
    }
    None
}

fn extract_alias_value(s: &str) -> Option<String> {
    let marker = "Alias::new(\"";
    let pos = s.find(marker)? + marker.len();
    let end = s[pos..].find('"')? + pos;
    Some(s[pos..end].to_string())
}

fn extract_name_value(s: &str) -> Option<String> {
    let marker = ".name(\"";
    let pos = s.find(marker)? + marker.len();
    let end = s[pos..].find('"')? + pos;
    Some(s[pos..end].to_string())
}

// ============================================================
// Type mapping heuristic (as before)
// ============================================================

fn seaorm_sql_type(line: &str) -> &str {
    if line.contains(".blob()")
        || line.contains(".binary(")
        || line.contains(".binary_len(")
        || line.contains(".var_binary(")
    {
        "BYTEA"
    } else if line.contains(".text()") {
        "TEXT"
    } else if line.contains(".char()") || line.contains(".char_len(") {
        "CHAR"
    } else if line.contains(".tiny_integer()") || line.contains(".small_integer()") {
        "SMALLINT"
    } else if line.contains(".big_unsigned()") {
        "BIGINT"
    } else if line.contains(".unsigned()") {
        "INTEGER"
    } else if line.contains(".big_integer()") {
        "BIGINT"
    } else if line.contains(".integer()") {
        "INTEGER"
    } else if line.contains(".float()") {
        "REAL"
    } else if line.contains(".double()") {
        "DOUBLE PRECISION"
    } else if line.contains(".decimal(") || line.contains(".decimal_len(") {
        "DECIMAL"
    } else if line.contains(".boolean()") {
        "BOOLEAN"
    } else if line.contains(".timestamp_tz()") || line.contains(".timestamp_with_time_zone()") {
        "TIMESTAMP WITH TIME ZONE"
    } else if line.contains(".timestamp()")
        || line.contains(".date_time()")
        || line.contains(".datetime()")
    {
        "TIMESTAMP"
    } else if line.contains(".date()") {
        "DATE"
    } else if line.contains(".time()") {
        "TIME"
    } else if line.contains(".uuid()") {
        "UUID"
    } else if line.contains(".json_binary()") {
        "JSONB"
    } else if line.contains(".json()") {
        "JSON"
    } else if line.contains(".inet()") {
        "INET"
    } else if line.contains(".cidr()") {
        "CIDR"
    } else if line.contains(".mac_address()") {
        "MACADDR"
    } else if line.contains(".interval()") {
        "INTERVAL"
    } else {
        "VARCHAR(255)"
    }
}
