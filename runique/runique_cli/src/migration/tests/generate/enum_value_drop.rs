use crate::migration::tests::*;

// ── Enum value DROP: WARNING (up) + re-add (down), Postgres only ──

#[test]
fn enum_value_drop_warns_up_and_readds_down_on_postgres() {
    let changes = Changes {
        enum_value_drops: vec![("status".into(), "Status".into(), "Legacy".into())],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains("WARNING: value \"Legacy\" removed"),
        "up warning:\n{up}"
    );
    assert!(
        down.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "down must check the real backend at runtime:\n{down}"
    );
    assert!(
        down.contains(".name(Alias::new(\"status\"))")
            && down.contains(".add_value(Alias::new(\"Legacy\"))")
            && down.contains(".if_not_exists()"),
        "down must re-add the value on Postgres via the Type::alter() builder, with the \
         type name lowercased to match the unquoted CREATE TYPE's case-folded identity:\n{down}"
    );
}
