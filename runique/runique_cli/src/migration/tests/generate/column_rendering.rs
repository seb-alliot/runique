use crate::migration::tests::*;

// ── Column rendering: defaults, unique, type matrix ──

const STR_DEFAULT: &str = r#"
use runique::prelude::*;
model! {
    Cfg,
    table: "cfg",
    pk: id => Pk,
    {
        role: text [nullable, default: "guest"],
    }
}
"#;

#[test]
fn string_literal_default_is_emitted() {
    let sql = generate_create_file(&parse_model(STR_DEFAULT));
    assert!(sql.contains(r#".default("guest")"#), "{sql}");
}

const UNIQUE_COL: &str = r#"
use runique::prelude::*;
model! {
    Acc,
    table: "acc",
    pk: id => Pk,
    {
        email: text [required, unique],
    }
}
"#;

#[test]
fn unique_required_column_renders_unique_key_not_null() {
    let sql = generate_create_file(&parse_model(UNIQUE_COL));
    assert!(
        sql.contains(
            r#".col(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())"#
        ),
        "{sql}"
    );
}

#[test]
fn semantic_types_map_to_seaorm_methods() {
    let sql = generate_create_file(&parse_model(TYPES_MODEL));
    for needle in [
        ".decimal_len(10, 2)",
        ".boolean()",
        ".uuid()",
        ".json()",
        ".big_integer()",
        ".double()",
    ] {
        assert!(sql.contains(needle), "missing `{needle}`:\n{sql}");
    }
}
