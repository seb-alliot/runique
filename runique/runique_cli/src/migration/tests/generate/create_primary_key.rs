use crate::migration::tests::*;

// ── CREATE TABLE: primary key types ──

fn model_pk(pk: &str) -> String {
    format!(
        "use runique::prelude::*;\nmodel! {{\n  Thing,\n  table: \"thing\",\n  pk: id => {pk},\n  {{\n    label: text,\n  }}\n}}\n"
    )
}

// `pk: id => Pk` defers to the global alias `runique::schema::Pk` — the generated
// SQL must follow big-pk/pk-uuid, see runique/tests/migration/test_parser.rs
// for the same check on the proc-macro side.
#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
#[test]
fn pk_generic_alias_is_autoincrement_integer_by_default() {
    let sql = generate_create_file(&parse_model(&model_pk("Pk")));
    assert!(
        sql.contains(
            r#".col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())"#
        ),
        "{sql}"
    );
}

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
#[test]
fn pk_generic_alias_is_autoincrement_big_integer_under_big_pk() {
    let sql = generate_create_file(&parse_model(&model_pk("Pk")));
    assert!(
        sql.contains(
            r#".col(ColumnDef::new(Alias::new("id")).big_integer().not_null().auto_increment().primary_key())"#
        ),
        "{sql}"
    );
}

#[cfg(feature = "pk-uuid")]
#[test]
fn pk_generic_alias_is_uuid_no_autoincrement_under_pk_uuid() {
    let sql = generate_create_file(&parse_model(&model_pk("Pk")));
    assert!(
        sql.contains(r#".col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())"#),
        "{sql}"
    );
    assert!(
        !sql.contains("auto_increment"),
        "generic Pk alias under pk-uuid must not auto_increment:\n{sql}"
    );
}

#[test]
fn pk_i64_is_big_integer_autoincrement() {
    let sql = generate_create_file(&parse_model(&model_pk("i64")));
    assert!(
        sql.contains(".big_integer().not_null().auto_increment().primary_key()"),
        "{sql}"
    );
}

#[test]
fn pk_uuid_has_no_autoincrement() {
    let sql = generate_create_file(&parse_model(&model_pk("uuid")));
    assert!(sql.contains(".uuid().not_null().primary_key()"), "{sql}");
    assert!(
        !sql.contains("auto_increment"),
        "uuid PK must not auto_increment:\n{sql}"
    );
}
