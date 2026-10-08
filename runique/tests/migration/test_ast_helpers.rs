//! The syn helpers that read migration files back. Written from cargo-mutants
//! survivors (2026-10-08): migration/utils/helpers.rs, paths.rs, parser_seaorm.rs.
use runique::migration::utils::helpers::{
    extract_alias_new_str, extract_alias_new_str_inner, extract_fk_action,
    extract_references_from_expr, extract_str_from_call, method_names_in_expr,
};

fn expr(src: &str) -> syn::Expr {
    syn::parse_str(src).unwrap()
}

#[test]
fn method_names_are_found_inside_function_calls_too() {
    let names = method_names_in_expr(&expr(
        r#"wrap(ColumnDef::new(Alias::new("n")).integer().not_null())"#,
    ));
    assert!(names.contains(&"integer".to_string()), "{names:?}");
    assert!(names.contains(&"not_null".to_string()), "{names:?}");
    assert!(method_names_in_expr(&expr("plain")).is_empty());
}

#[test]
fn the_first_string_is_found_through_a_method_chain() {
    assert_eq!(
        extract_str_from_call(&expr(r#"Alias::new("users").to_owned()"#)).as_deref(),
        Some("users")
    );
    assert_eq!(
        extract_str_from_call(&expr("Alias::new(name).to_owned()")),
        None
    );
}

#[test]
fn references_without_a_table_name_are_none_never_a_panic() {
    assert_eq!(
        extract_references_from_expr(&expr(r#"col.references("users")"#)),
        Some(("users".to_string(), "id".to_string()))
    );
    assert_eq!(
        extract_references_from_expr(&expr("col.references(table)")),
        None
    );
}

#[test]
fn fk_actions_are_found_in_receivers_and_in_arguments() {
    let in_receiver = expr("ForeignKey::create().on_delete(ForeignKeyAction::Cascade).to_owned()");
    assert_eq!(extract_fk_action(&in_receiver, "on_delete"), "Cascade");
    let in_argument = expr("wrap.call(fk.on_delete(ForeignKeyAction::SetNull))");
    assert_eq!(extract_fk_action(&in_argument, "on_delete"), "SetNull");
    assert_eq!(extract_fk_action(&in_argument, "on_update"), "NoAction");
}

#[test]
fn only_alias_new_gives_a_name() {
    assert_eq!(
        extract_alias_new_str(&expr(r#"t.table(Alias::new("posts"))"#)).as_deref(),
        Some("posts")
    );
    assert_eq!(
        extract_alias_new_str(&expr(r#"t.table(Other::new("posts"))"#)),
        None
    );
    assert_eq!(
        extract_alias_new_str_inner(&expr(r#"Alias::new("posts")"#)).as_deref(),
        Some("posts")
    );
    assert_eq!(
        extract_alias_new_str_inner(&expr(r#"Other::new("posts")"#)),
        None
    );
}

#[test]
fn extension_snapshots_live_under_snapshots_runique() {
    use runique::migration::utils::paths::{extend_snapshot_dir, extend_snapshot_file_path};
    assert_eq!(
        extend_snapshot_dir("migration/src"),
        "migration/src/snapshots/runique"
    );
    assert_eq!(
        extend_snapshot_file_path("migration/src", "eihwaz_users"),
        "migration/src/snapshots/runique/eihwaz_users.rs"
    );
}

/// A foreign key whose `from` has a single argument is skipped, without a
/// panic (`mc.args[1]` would be out of bounds).
#[test]
fn a_one_argument_from_does_not_panic() {
    use runique::migration::utils::parser_seaorm::parse_seaorm_source;
    let src = r#"
use sea_orm_migration::prelude::*;
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("posts"))
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().primary_key())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("posts_author_fkey")
                    .from(Alias::new("author_id"))
                    .to(Alias::new("users"), Alias::new("id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
}
"#;
    let parsed = parse_seaorm_source(src).expect("parsed");
    assert_eq!(parsed.table_name, "posts");
}

/// Integer-backed enums map to the integer column of their size.
#[test]
fn integer_enums_get_their_integer_column_type() {
    use runique::migration::utils::parser_builder::parse_schema_from_source;
    let src = r#"model! { Item, table: "items", pk: id => i32,
        enums: { Tiny: i8 [A = 1, B = 2], Huge: i64 [X = 1, Y = 2] },
        { tiny: choice [enum(Tiny), required], huge: choice [enum(Huge), required] } }"#;
    let schema = parse_schema_from_source(src).unwrap().unwrap().1;
    let ty = |name: &str| {
        schema
            .columns
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .col_type
            .clone()
    };
    assert_eq!(ty("tiny"), "TinyInteger");
    assert_eq!(ty("huge"), "BigInteger");
}
