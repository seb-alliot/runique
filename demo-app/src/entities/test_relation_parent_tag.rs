//! Test coverage entity — manually-declared junction table for the
//! `TestRelationParent` <-> `TestRelationTag` many-to-many. `many_to_many:`
//! only generates the SeaORM navigation code (no DB constraint of its own,
//! see docs/*/model/dsl/dsl.md) — the pivot table and its FKs must exist as
//! an ordinary entity, exactly like this one.
//!
//! FK columns are named `{target_model_snake_case}_id` — `belongs_to` names
//! its generated `Relation` variant after the target model, while
//! `many_to_many`'s `via` derives the variant it looks up on this through
//! entity from the column name itself (strips `_id`, pascal-cases). The two
//! schemes only agree when the column follows this exact convention.
use runique::prelude::*;

model! {
    TestRelationParentTag,
    table: "test_relation_parent_tag",
    pk: id => Pk,
    {
        test_relation_parent_id: Pk [required],
        test_relation_tag_id:    Pk [required],
    },
    relations: {
        belongs_to: TestRelationParent via test_relation_parent_id [cascade],
        belongs_to: TestRelationTag via test_relation_tag_id [cascade],
    },
    meta: {
        unique_together: [(test_relation_parent_id, test_relation_tag_id)],
    }
}
