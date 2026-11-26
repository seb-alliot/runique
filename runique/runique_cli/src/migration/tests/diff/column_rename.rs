use crate::migration::tests::*;

// ── Column rename via `[renamed_from: "old"]` ──

const COL_BEFORE: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        job_title: text [nullable],
        age: int [nullable],
    }
}
"#;

const COL_RENAMED: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        title: text [nullable, renamed_from: "job_title"],
        age: int [nullable],
    }
}
"#;

const COL_NO_HINT: &str = r#"
use runique::prelude::*;
model! {
    Person,
    table: "person",
    pk: id => Pk,
    {
        title: text [nullable],
        age: int [nullable],
    }
}
"#;

#[test]
fn column_rename_is_detected_via_hint() {
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    assert_eq!(
        changes.renamed_columns,
        vec![("job_title".to_string(), "title".to_string())]
    );
    assert!(
        changes.added_columns.is_empty(),
        "rename must not add: {:?}",
        changes.added_columns
    );
    assert!(
        changes.dropped_columns.is_empty(),
        "rename must not drop: {:?}",
        changes.dropped_columns
    );
}

#[test]
fn column_rename_without_hint_is_drop_add() {
    // No `renamed_from` → the diff cannot guess intent: it's a destructive drop + add.
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_NO_HINT));
    assert!(changes.renamed_columns.is_empty());
    assert!(changes.added_columns.iter().any(|c| c.name == "title"));
    assert!(
        changes
            .dropped_columns
            .iter()
            .any(|c| c.name == "job_title")
    );
}

#[test]
fn column_rename_sql_is_portable_across_engines() {
    // RENAME COLUMN is supported by PG / MySQL / SQLite → same builder for all.
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains(r#".rename_column(Alias::new("job_title"), Alias::new("title"))"#),
        "RENAME COLUMN missing:\n{sql}"
    );
    assert!(!sql.contains("drop_column"), "rename must not DROP:\n{sql}");
    assert!(!sql.contains("add_column"), "rename must not ADD:\n{sql}");
}

#[test]
fn column_rename_down_reverses() {
    let changes = diff_schemas(&parse_model(COL_BEFORE), &parse_model(COL_RENAMED));
    let sql = generate_alter_file(&changes);
    let down = sql.split("async fn down").nth(1).unwrap_or("");
    assert!(
        down.contains(r#".rename_column(Alias::new("title"), Alias::new("job_title"))"#),
        "down must reverse the rename:\n{sql}"
    );
}

#[test]
fn stale_renamed_from_falls_back_to_add() {
    // If the old column still exists (stale hint), do NOT rename — treat `title` as a new column.
    let changes = diff_schemas(&parse_model(COL_NO_HINT), &parse_model(COL_RENAMED));
    // prev has `title`+`age`, curr has `title`(renamed_from job_title)+`age`.
    // job_title is absent from prev → guard fails → no rename.
    assert!(
        changes.renamed_columns.is_empty(),
        "stale hint must not rename: {:?}",
        changes.renamed_columns
    );
}

const EXTEND_COL_BEFORE: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        job_title: text [nullable],
    }
}
"#;

const EXTEND_COL_RENAMED: &str = r#"
use runique::prelude::*;
extend! {
    table: "eihwaz_users",
    fields: {
        title: text [nullable, renamed_from: "job_title"],
    }
}
"#;

#[test]
fn extend_column_rename_is_detected() {
    let prev = parse_extend_blocks_from_source(EXTEND_COL_BEFORE)
        .unwrap()
        .remove(0);
    let curr = parse_extend_blocks_from_source(EXTEND_COL_RENAMED)
        .unwrap()
        .remove(0);
    let changes = diff_schemas(&prev, &curr);
    assert_eq!(
        changes.renamed_columns,
        vec![("job_title".to_string(), "title".to_string())]
    );
    assert!(changes.added_columns.is_empty());
    assert!(changes.dropped_columns.is_empty());
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains(r#".rename_column(Alias::new("job_title"), Alias::new("title"))"#),
        "extend rename must emit RENAME COLUMN:\n{sql}"
    );
}
