use crate::migration::tests::*;

// ── Snapshot round-trip (the golden stability test) ──

#[test]
fn snapshot_default_does_not_set_has_default_now() {
    // the SeaORM parser (`parser/seaorm.rs`) must NOT treat a literal `.default(0)` as a CURRENT_TIMESTAMP default.
    let schema = parse_model(BLOG_SRC);
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    assert!(
        !col(&reparsed, "view_count").has_default_now,
        "literal default must not be read back as a timestamp default"
    );
}

#[test]
fn snapshot_round_trip_is_stable() {
    // The invariant behind "makemigrations twice = no changes".
    let schema = parse_model(BLOG_SRC);
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    let changes = diff_schemas(&reparsed, &schema);
    assert!(
        changes.is_empty(),
        "round-trip not stable, spurious changes: {changes:?}"
    );
}
