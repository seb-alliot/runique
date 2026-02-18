use anyhow::{Context, Result};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend};
use std::fs;
use std::path::Path;

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

async fn execute_down_block(source: &str, db: &DatabaseConnection) -> Result<()> {
    let backend = db.get_database_backend();
    let statements = extract_down_statements(source, backend);

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

fn extract_down_statements(source: &str, backend: DbBackend) -> Vec<String> {
    let mut statements = Vec::new();
    let mut in_down = false;
    let mut brace_depth: i32 = 0;

    let table_name = extract_table_from_source(source).unwrap_or_default();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.contains("async fn down") {
            in_down = true;
            brace_depth = 0;
            continue;
        }

        if !in_down {
            continue;
        }

        brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;

        if brace_depth < 0 {
            break;
        }

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

        if trimmed.contains(".drop_table(") && trimmed.contains("Alias::new(\"") {
            if let Some(t) = extract_alias_value(trimmed) {
                let sql = match backend {
                    DbBackend::Postgres => format!("DROP TABLE IF EXISTS \"{}\";", t),
                    _ => format!("DROP TABLE IF EXISTS `{}`;", t),
                };
                statements.push(sql);
            }
        }
    }

    statements
}

fn extract_table_from_source(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(".table(Alias::new(\"") {
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

fn seaorm_sql_type(line: &str) -> &str {
    if line.contains(".text()") {
        "TEXT"
    } else if line.contains(".big_integer()") {
        "BIGINT"
    } else if line.contains(".integer()") {
        "INTEGER"
    } else if line.contains(".boolean()") {
        "BOOLEAN"
    } else if line.contains(".date_time()") {
        "TIMESTAMP"
    } else if line.contains(".uuid()") {
        "UUID"
    } else if line.contains(".json()") {
        "JSON"
    } else {
        "VARCHAR(255)"
    }
}
