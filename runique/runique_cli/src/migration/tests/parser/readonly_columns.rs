use crate::migration::tests::*;

// ── Ignored columns (readonly) are excluded from migrations ──

const IGNORED_COL: &str = r#"
use runique::prelude::*;
model! {
    Sec,
    table: "sec",
    pk: id => Pk,
    {
        visible: text [nullable],
        note: text [nullable, readonly],
    }
}
"#;

#[test]
fn readonly_column_is_excluded_from_migration() {
    let schema = parse_model(IGNORED_COL);
    assert!(
        col(&schema, "note").ignored,
        "readonly column must be flagged ignored"
    );
    let sql = generate_create_file(&schema);
    assert!(
        sql.contains(r#"Alias::new("visible")"#),
        "visible column expected:\n{sql}"
    );
    assert!(
        !sql.contains(r#"Alias::new("note")"#),
        "ignored column must not appear in the migration:\n{sql}"
    );
}
