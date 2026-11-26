use crate::migration::tests::*;

// ── CREATE TABLE: timestamps per engine ──

const TS_MODEL: &str = r#"
use runique::prelude::*;
model! {
    Event,
    table: "event",
    pk: id => Pk,
    {
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
"#;

#[test]
fn auto_now_columns_default_to_current_timestamp() {
    let schema = parse_model(TS_MODEL);
    assert!(col(&schema, "created_at").has_default_now);
    assert!(col(&schema, "updated_at").has_default_now);
    let sql = generate_create_file(&schema);
    assert!(sql.contains(".default(Expr::current_timestamp())"), "{sql}");
}

#[test]
fn a_column_name_alone_decides_nothing() {
    let schema = parse_model(
        r#"model! { E, table: "e", pk: id => i32, {
            created_at: datetime [nullable],
            cache_key: text,
        } }"#,
    );
    assert!(!col(&schema, "created_at").has_default_now);
    assert!(!col(&schema, "cache_key").ignored);
    let sql = generate_create_file(&schema);
    assert!(!sql.contains("current_timestamp"), "{sql}");
}

// `updated_at` is the entity's job (`[auto_now_update]` → `before_save`, the
// same on every engine): the migration adds no trigger, function or
// `ON UPDATE` clause, and no raw SQL at all.
#[test]
fn updated_at_is_left_to_the_entity() {
    let sql = generate_create_file(&parse_model(TS_MODEL));
    for raw in [
        "TRIGGER",
        "plpgsql",
        "ON UPDATE CURRENT_TIMESTAMP",
        "execute_unprepared",
    ] {
        assert!(!sql.contains(raw), "{raw}:\n{sql}");
    }
}
