use crate::migration::tests::*;

// ── A column renamed and modified in the same run ──

const JOB_TITLE: &str = "job_title: text [nullable]";

// The rename and the modification are both kept: the column is compared to
// what it was under its former name, and enters the list under its new one.
#[test]
fn a_renamed_column_that_also_changed_is_modified_under_its_new_name() {
    let changes = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [required, renamed_from: "job_title"]"#),
    );

    assert_eq!(
        changes.renamed_columns,
        [("job_title".to_string(), "title".to_string())]
    );
    assert!(changes.added_columns.is_empty() && changes.dropped_columns.is_empty());
    assert_eq!(changes.modified_columns.len(), 1);
    let (was, now) = &changes.modified_columns[0];
    assert_eq!((was.name.as_str(), was.nullable), ("title", true));
    assert_eq!((now.name.as_str(), now.nullable), ("title", false));
}

// Every part of a definition counts, as for a column that keeps its name.
#[test]
fn every_kind_of_change_is_seen_on_a_renamed_column() {
    for (field, what) in [
        (
            r#"title: text [required, renamed_from: "job_title"]"#,
            "nullable",
        ),
        (
            r#"title: text [nullable, unique, renamed_from: "job_title"]"#,
            "unique",
        ),
        (
            r#"title: text [nullable, max_length: 40, renamed_from: "job_title"]"#,
            "length",
        ),
        (
            r#"title: int [nullable, renamed_from: "job_title"]"#,
            "type",
        ),
    ] {
        let changes = diff_schemas(&person(JOB_TITLE), &person(field));
        assert_eq!(changes.renamed_columns.len(), 1, "{what}");
        assert_eq!(changes.modified_columns.len(), 1, "{what}");
        assert_eq!(changes.modified_columns[0].1.name, "title", "{what}");
    }
}

// The other side: renamed and nothing else, no modification.
#[test]
fn a_column_only_renamed_is_not_modified() {
    let changes = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [nullable, renamed_from: "job_title"]"#),
    );
    assert_eq!(changes.renamed_columns.len(), 1);
    assert!(
        changes.modified_columns.is_empty(),
        "{:?}",
        changes.modified_columns
    );
}

// A column modified without being renamed is still seen, under its own name.
#[test]
fn a_column_only_modified_is_still_modified() {
    let changes = diff_schemas(&person(JOB_TITLE), &person("job_title: text [required]"));
    assert!(changes.renamed_columns.is_empty());
    assert_eq!(changes.modified_columns.len(), 1);
    let (was, now) = &changes.modified_columns[0];
    assert_eq!(
        (was.name.as_str(), now.name.as_str()),
        ("job_title", "job_title")
    );
}

// A `renamed_from` that doesn't hold renames nothing: the column is new, and
// is not compared to another one.
#[test]
fn a_stale_rename_compares_nothing() {
    let changes = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [required, renamed_from: "never_there"]"#),
    );
    assert!(changes.renamed_columns.is_empty());
    assert!(
        changes.modified_columns.is_empty(),
        "{:?}",
        changes.modified_columns
    );
    assert!(changes.added_columns.iter().any(|c| c.name == "title"));
}

// Two columns renamed, one of them modified: only that one is.
#[test]
fn only_the_renamed_column_that_changed_is_modified() {
    let before = parse_model(
        r#"model! { Person, table: "person", pk: id => i32, { job_title: text [nullable], town: text [nullable] } }"#,
    );
    let after = parse_model(
        r#"model! { Person, table: "person", pk: id => i32, {
            title: text [nullable, renamed_from: "job_title"],
            city: text [required, renamed_from: "town"] } }"#,
    );
    let changes = diff_schemas(&before, &after);

    assert_eq!(changes.renamed_columns.len(), 2);
    assert_eq!(changes.modified_columns.len(), 1);
    assert_eq!(changes.modified_columns[0].1.name, "city");
}

// The migration renames first, then modifies the column under its new name;
// going back, it undoes the modification, still under the new name, then the
// rename.
#[test]
fn the_migration_renames_then_modifies_and_goes_back_in_reverse() {
    let changes = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [required, renamed_from: "job_title"]"#),
    );
    let file = generate_alter_file(&changes);
    let (up, down) = file.split_once("async fn down").expect("a down");

    let rename = up
        .find(r#".rename_column(Alias::new("job_title"), Alias::new("title"))"#)
        .expect("the rename");
    let modify = up
        .find(r#".modify_column(ColumnDef::new(Alias::new("title"))"#)
        .expect("the modification, under the new name");
    assert!(rename < modify, "{up}");
    assert!(up[modify..].contains(".not_null()"), "{up}");
    assert!(
        !up.contains(r#"ColumnDef::new(Alias::new("job_title"))"#),
        "{up}"
    );

    let modify_back = down
        .find(r#".modify_column(ColumnDef::new(Alias::new("title"))"#)
        .expect("the modification undone, under the new name");
    let rename_back = down
        .find(r#".rename_column(Alias::new("title"), Alias::new("job_title"))"#)
        .expect("the rename undone");
    assert!(modify_back < rename_back, "{down}");
    assert!(
        !down.contains(r#"ColumnDef::new(Alias::new("job_title"))"#),
        "{down}"
    );
}

// The modification of a renamed column is guarded like any other: becoming
// required asks for `--force`, under the name the column has now.
#[test]
fn the_modification_of_a_renamed_column_reaches_the_destructive_guard() {
    let changes = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [required, renamed_from: "job_title"]"#),
    );
    let messages = collect_destructive_messages(std::slice::from_ref(&changes));
    assert!(
        messages
            .iter()
            .any(|m| m.contains("person.title: nullable -> not_null")),
        "{messages:?}"
    );

    let renamed_only = diff_schemas(
        &person(JOB_TITLE),
        &person(r#"title: text [nullable, renamed_from: "job_title"]"#),
    );
    assert!(collect_destructive_messages(std::slice::from_ref(&renamed_only)).is_empty());
}
