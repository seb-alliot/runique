use crate::db::DatabaseEngine;
use crate::migration::tests::*;

// ── Enum value DROP: WARNING (up) + re-add (down), Postgres only ──

#[test]
fn enum_value_drop_warns_up_and_readds_down_on_postgres() {
    let changes = EnumChanges {
        name: "status".into(),
        dropped_values: vec!["Legacy".into()],
        ..EnumChanges::default()
    };
    let sql = generate_enum_file(&changes, &DatabaseEngine::PostgreSQL);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains("WARNING: value \"Legacy\" removed"),
        "up warning:\n{up}"
    );
    assert!(
        down.contains(
            r#"runique_cli::migration::runtime::add_enum_value(manager, "status", "Legacy").await?;"#
        ),
        "down must give the type its value again, through the runtime:\n{down}"
    );
    // Where the column is text there is no list to give the value to.
    for engine in [DatabaseEngine::MySQL, DatabaseEngine::SQLite] {
        let sql = generate_enum_file(&changes, &engine);
        let (up, down) = up_down(&sql);
        assert!(up.contains("WARNING: value \"Legacy\" removed"), "{up}");
        assert!(!down.contains("add_enum_value"), "{down}");
    }
}
