use crate::migration::tests::*;

// ── ALTER: add / drop index (up + down) ──

#[test]
fn alter_add_unique_index_up_creates_down_drops() {
    let changes = Changes {
        added_indexes: vec![ParsedIndex {
            name: "idx_t_email".into(),
            columns: vec!["email".into()],
            unique: true,
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(up.contains(".create_index("), "up:\n{up}");
    assert!(up.contains(r#".name("idx_t_email")"#), "up:\n{up}");
    assert!(up.contains(".unique()"), "unique index:\n{up}");
    assert!(
        down.contains(r#".drop_index(Index::drop().name("idx_t_email")"#),
        "down:\n{down}"
    );
}

#[test]
fn alter_drop_index_up_drops_down_recreates() {
    let changes = Changes {
        dropped_indexes: vec![ParsedIndex {
            name: "idx_t_slug".into(),
            columns: vec!["slug".into()],
            unique: false,
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".drop_index(Index::drop().name("idx_t_slug")"#),
        "up:\n{up}"
    );
    assert!(down.contains(".create_index("), "down:\n{down}");
    assert!(
        !down.contains(".unique()"),
        "non-unique index must not be unique on recreate:\n{down}"
    );
}
