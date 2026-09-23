//! Test coverage entity — the `has_one` side of `TestRelationParent`.
//! `parent_id` is `unique` so the FK actually enforces a 1-1 relationship,
//! not just 1-N.
use runique::prelude::*;

model! {
    TestRelationProfile,
    table: "test_relation_profile",
    pk: id => Pk,
    {
        parent_id: Pk [required, unique],
        bio:       text,
    },
    relations: {
        belongs_to: TestRelationParent via parent_id [cascade],
    }
}
