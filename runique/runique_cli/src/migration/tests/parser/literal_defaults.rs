use crate::migration::tests::*;

// ── Literal defaults ──

#[test]
fn literal_default_is_parsed() {
    let schema = parse_model(BLOG_SRC);
    assert_eq!(
        col(&schema, "view_count").default_value.as_deref(),
        Some("0")
    );
    assert_eq!(
        col(&schema, "is_active").default_value.as_deref(),
        Some("true")
    );
}

#[test]
fn literal_default_is_emitted_in_create() {
    let sql = generate_create_file(&parse_model(BLOG_SRC));
    assert!(sql.contains(".default(0)"), "int default missing:\n{sql}");
    assert!(
        sql.contains(".default(true)"),
        "bool default missing:\n{sql}"
    );
}

#[test]
fn columns_are_not_null_unless_declared_nullable() {
    let schema = parse_model(
        r#"model! { T, table: "t", pk: id => i32, {
            flag: bool [default: true],
            name: text,
            count: int [required],
            note: int [nullable],
        } }"#,
    );
    assert!(!col(&schema, "flag").nullable);
    assert!(!col(&schema, "name").nullable);
    assert!(!col(&schema, "count").nullable);
    assert!(col(&schema, "note").nullable);
}
