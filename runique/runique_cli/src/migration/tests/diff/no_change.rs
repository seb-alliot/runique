use crate::migration::tests::*;

// ── No-op diff ──

#[test]
fn identical_schemas_produce_no_changes() {
    let changes = diff_schemas(&parse_model(TYPES_MODEL), &parse_model(TYPES_MODEL));
    assert!(
        changes.is_empty(),
        "identical schemas must diff to empty: {changes:?}"
    );
}
