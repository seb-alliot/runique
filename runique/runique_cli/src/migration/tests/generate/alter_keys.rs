use crate::migration::tests::*;

// ── ALTER: add / drop FK (up + down) ──

#[test]
fn alter_add_fk_up_creates_down_drops() {
    let changes = Changes {
        added_fks: vec![fk("post_id", "post")],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(up.contains("runtime::add_foreign_key("), "up:\n{up}");
    assert!(up.contains(r#".name("t_post_id_post_fkey")"#), "up:\n{up}");
    assert!(
        down.contains(r#"runtime::drop_foreign_key(manager, "t", "t_post_id_post_fkey")"#),
        "down:\n{down}"
    );
    key_guard_comes_first(up, down);
}

/// Both bodies of a migration that changes keys start by asking the engine
/// whether it can: nothing of the migration runs before.
fn key_guard_comes_first(up: &str, down: &str) {
    let guard = r#"runique_cli::migration::runtime::key_changes_supported(manager, "t")?;"#;
    for body in [up, down] {
        let first = body
            .lines()
            .map(str::trim)
            // The first statement: the signature line is skipped.
            .find(|line| line.contains("manager") && !line.contains("&self"))
            .unwrap_or_default();
        assert_eq!(first, guard, "{body}");
        assert_eq!(body.matches("key_changes_supported").count(), 1, "{body}");
    }
}

// A migration that changes no key asks nothing: it must keep running on
// SQLite as before.
#[test]
fn an_alter_without_a_key_change_has_no_key_guard() {
    let changes = Changes {
        added_columns: vec![ParsedColumn {
            name: "bio".into(),
            col_type: crate::migration::ColumnType::String,
            nullable: true,
            ..ParsedColumn::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    assert!(!sql.contains("key_changes_supported"), "{sql}");
}

#[test]
fn alter_drop_fk_up_drops_down_creates() {
    let changes = Changes {
        dropped_fks: vec![fk("post_id", "post")],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#"runtime::drop_foreign_key(manager, "t", "t_post_id_post_fkey")"#),
        "up:\n{up}"
    );
    assert!(down.contains("runtime::add_foreign_key("), "down:\n{down}");
    key_guard_comes_first(up, down);
}
