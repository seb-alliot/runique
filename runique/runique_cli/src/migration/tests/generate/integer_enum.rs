use crate::migration::tests::*;

// ── i32-backed enum: integer column, never CREATE TYPE ──

const ENUM_I32: &str = r#"
use runique::prelude::*;
model! {
    Task,
    table: "task",
    pk: id => Pk,
    enums: {
        Priority: i32 [Low=1, Mid=2, High=3],
    },
    {
        priority: choice [enum(Priority), required],
    }
}
"#;

#[test]
fn i32_backed_enum_is_integer_column_without_create_type() {
    let schema = parse_model(ENUM_I32);
    let prio = col(&schema, "priority");
    assert_eq!(prio.col_type, "Integer");
    assert!(
        prio.enum_string_values.is_empty(),
        "i32-backed enum carries no string variants"
    );
    let pg = generate_create_file(&schema);
    assert!(
        !pg.contains("CREATE TYPE"),
        "i32 enum must not CREATE TYPE:\n{pg}"
    );
    assert!(
        pg.contains(r#".col(ColumnDef::new(Alias::new("priority")).integer()"#),
        "i32 enum is an integer column:\n{pg}"
    );
}
