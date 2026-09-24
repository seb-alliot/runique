//! Test coverage entity — manually-declared junction table for the
//! `TestRelationParent` <-> `TestRelationTag` many-to-many. `many_to_many:`
//! only generates the SeaORM navigation code (no DB constraint of its own,
//! see docs/*/model/dsl/dsl.md) — the pivot table and its FKs must exist as
//! an ordinary entity, exactly like this one.
//!
//! FK columns are named `parent_id`/`tag_id` rather than the fuller
//! `test_relation_parent_id`/`test_relation_tag_id` — combined with the table
//! name, the longer form pushes the generated `unique_together` index name
//! past MariaDB/MySQL's 64-character identifier limit (`makemigrations`
//! fails loudly rather than silently truncating it, see `helpers.rs`'s
//! `check_sql_identifier_len`). Real schemas rarely stack this many
//! long, repeated name segments — this is a test-only naming quirk.
use runique::prelude::*;

model! {
    TestRelationParentTag,
    table: "test_relation_parent_tag",
    pk: id => Pk,
    {
        parent_id: Pk [required],
        tag_id:    Pk [required],
    },
    relations: {
        belongs_to: TestRelationParent via parent_id [cascade],
        belongs_to: TestRelationTag via tag_id [cascade],
    },
    meta: {
        unique_together: [(parent_id, tag_id)],
    }
}
