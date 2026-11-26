use crate::migration::tests::*;

// ── Topological sort: referenced tables created first ──

fn new_table(name: &str, fks: Vec<ParsedFk>) -> Changes {
    Changes {
        table_name: name.into(),
        table: crate::migration::TableChange::Created,
        added_fks: fks,
        ..empty_changes()
    }
}

#[test]
fn topological_sort_creates_referenced_table_first() {
    // `comment` has a FK to `post`; both are new → `post` must be created first.
    let sorted = topological_sort_changes(vec![
        new_table("comment", vec![fk("post_id", "post")]),
        new_table("post", vec![]),
    ]);
    let order: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    let post = order.iter().position(|t| *t == "post").unwrap();
    let comment = order.iter().position(|t| *t == "comment").unwrap();
    assert!(
        post < comment,
        "referenced table must come first: {order:?}"
    );
}

#[test]
fn topological_sort_orders_a_fk_chain() {
    // a → b → c : creation order must be c, then b, then a.
    let sorted = topological_sort_changes(vec![
        new_table("a", vec![fk("b_id", "b")]),
        new_table("b", vec![fk("c_id", "c")]),
        new_table("c", vec![]),
    ]);
    let order: Vec<&str> = sorted.iter().map(|c| c.table_name.as_str()).collect();
    let pos = |t: &str| order.iter().position(|x| *x == t).unwrap();
    assert!(
        pos("c") < pos("b") && pos("b") < pos("a"),
        "chain order wrong: {order:?}"
    );
}

#[test]
fn topological_sort_ignores_fk_to_existing_table() {
    // A FK to a table NOT created in this batch does not constrain ordering (no panic, kept).
    let sorted = topological_sort_changes(vec![new_table("comment", vec![fk("user_id", "users")])]);
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0].table_name, "comment");
}
