//! Test coverage entity — exercises `has_many`/`has_one`/`many_to_many` in
//! `relations:`, never used by any other real entity in this project.
use runique::prelude::*;

model! {
    TestRelationParent,
    table: "test_relation_parent",
    pk: id => Pk,
    {
        name: text [required],
    },
    relations: {
        has_many: TestRelationChild,
        has_one: TestRelationProfile,
        many_to_many: TestRelationTag through TestRelationParentTag via parent_id,
    }
}
