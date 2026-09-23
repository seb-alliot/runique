//! Test coverage entity — the other logical side of the `many_to_many`
//! declared on `TestRelationParent`, navigated through `TestRelationParentTag`.
use runique::prelude::*;

model! {
    TestRelationTag,
    table: "test_relation_tag",
    pk: id => Pk,
    {
        name: text [required, unique],
    },
    relations: {
        many_to_many: TestRelationParent through TestRelationParentTag via test_relation_tag_id,
    }
}
