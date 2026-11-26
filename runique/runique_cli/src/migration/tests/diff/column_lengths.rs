use crate::migration::tests::*;

// ── Column lengths ──

fn titled(len: u32) -> ParsedSchema {
    parse_model(&format!(
        r#"model! {{ Post, table: "post", pk: id => i32, {{ title: text [required, max_length: {len}] }} }}"#
    ))
}

fn priced(max_digits: u32, decimal_places: u32) -> ParsedSchema {
    parse_model(&format!(
        r#"model! {{ Item, table: "item", pk: id => i32, {{ price: decimal [required, max_digits: {max_digits}, decimal_places: {decimal_places}] }} }}"#
    ))
}

#[test]
fn snapshot_round_trips_the_decimal_precision() {
    let schema = priced(10, 2);
    let reparsed = parse_seaorm_source(&generate_snapshot_file(&schema)).unwrap();
    assert_eq!(col(&reparsed, "price").decimal_len, Some((10, 2)));
    assert!(diff_schemas(&reparsed, &schema).is_empty());
}

#[test]
fn a_precision_change_is_an_alter() {
    let changes = diff_schemas(&priced(10, 2), &priced(12, 2));
    assert_eq!(changes.modified_columns.len(), 1);
    let alter = generate_alter_file(&changes);
    assert!(alter.contains(".decimal_len(12, 2)"), "{alter}");
    assert!(collect_destructive_messages(&[changes]).is_empty());
}

#[test]
fn a_precision_losing_digits_needs_force() {
    let changes = diff_schemas(&priced(10, 2), &priced(10, 1));
    let messages = collect_destructive_messages(&[changes]);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("decimal (10, 2) -> (10, 1)")),
        "{messages:?}"
    );
}

#[test]
fn a_first_precision_on_an_old_decimal_needs_force() {
    let mut old = priced(10, 2);
    old.columns[0].decimal_len = None;
    let changes = diff_schemas(&old, &priced(10, 2));
    assert_eq!(changes.modified_columns.len(), 1);
    let messages = collect_destructive_messages(&[changes]);
    assert!(
        messages
            .iter()
            .any(|m| m.contains("decimal default -> (10, 2)")),
        "{messages:?}"
    );
}

#[test]
fn declared_length_reaches_the_create_file() {
    assert_eq!(col(&titled(80), "title").max_length, Some(80));
    let sql = generate_create_file(&titled(80));
    assert!(sql.contains(".string_len(80)"), "{sql}");
}

#[test]
fn email_and_binary_columns_get_their_length() {
    let schema = parse_model(
        r#"model! { T, table: "t", pk: id => i32, { mail: email [required], raw: binary [required], bytes: var_binary [required, max_length: 16] } }"#,
    );
    let sql = generate_create_file(&schema);
    assert!(sql.contains(".string_len(254)"), "{sql}");
    assert!(sql.contains(".binary_len(255)"), "{sql}");
    assert!(sql.contains(".var_binary(16)"), "{sql}");
}

#[test]
fn snapshot_round_trips_the_length() {
    let schema = titled(80);
    let reparsed = parse_seaorm_source(&generate_snapshot_file(&schema)).unwrap();
    assert_eq!(col(&reparsed, "title").max_length, Some(80));
    assert!(diff_schemas(&reparsed, &schema).is_empty());
}

#[test]
fn growing_a_length_is_a_plain_alter() {
    let changes = diff_schemas(&titled(80), &titled(120));
    assert_eq!(changes.modified_columns.len(), 1);
    let alter = generate_alter_file(&changes);
    assert!(alter.contains(".string_len(120)"), "{alter}");
    assert!(collect_destructive_messages(&[changes]).is_empty());
}

#[test]
fn shrinking_a_length_needs_force() {
    let changes = diff_schemas(&titled(120), &titled(80));
    let messages = collect_destructive_messages(&[changes]);
    assert!(
        messages.iter().any(|m| m.contains("length 120 -> 80")),
        "{messages:?}"
    );
}

#[test]
fn a_snapshot_from_before_lengths_takes_the_model_lengths() {
    let current = titled(80);
    let old = generate_snapshot_file(&parse_model(
        r#"model! { Post, table: "post", pk: id => i32, { title: text [required] } }"#,
    ));
    let old = old
        .strip_prefix(crate::migration::SNAPSHOT_LENGTHS_MARKER)
        .unwrap()
        .to_string();
    let dir = std::env::temp_dir().join(format!("runique_len_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("post.rs");
    std::fs::write(&path, old).unwrap();

    let previous = crate::migration::steps::parser::files::snapshot::previous_snapshot(
        path.to_str().unwrap(),
        &current,
    )
    .unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert!(diff_schemas(&previous, &current).is_empty());
}
