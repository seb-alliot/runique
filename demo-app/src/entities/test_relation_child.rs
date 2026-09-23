//! Test coverage entity — the `has_many` side of `TestRelationParent`.
use runique::prelude::*;

model! {
    TestRelationChild,
    table: "test_relation_child",
    pk: id => Pk,
    {
        parent_id: Pk [required],
        label:     text [required],
    },
    relations: {
        belongs_to: TestRelationParent via parent_id [cascade],
    }
}
