// Regression test: `Pk` on a `fk()` field under `pk-uuid`.
//
// `Pk` resolves to `FormFieldKind::Uuid` under this feature (derive_form
// parser.rs:678-691). The attribute-validity gate used to only allow `fk()`
// on `Int | Bigint` (parser.rs:963-965), rejecting this at compile time —
// found 2026-09-02 while about to document `Pk` as the standard for FK
// fields; the "5 fields fixed in demo-app" claim in the CHANGELOG never
// actually exercised `pk-uuid` (demo-app runs under `postgres` only).
#[cfg(feature = "pk-uuid")]
mod scratch_parent_uuid_check {
    use runique::prelude::*;

    model! {
        ScratchParent,
        table: "scratch_parent_uuid_check",
        pk: id => Pk,
        {
            name: text [required],
        }
    }
}

#[cfg(feature = "pk-uuid")]
mod scratch_child {
    use runique::prelude::*;

    model! {
        ScratchChild,
        table: "scratch_child_uuid_check",
        pk: id => Pk,
        {
            parent_id: Pk [required, fk(scratch_parent_uuid_check.id, cascade)],
        }
    }

    #[test]
    fn test_pk_fk_field_resolves_to_uuid_column() {
        use runique::sea_orm::ColumnTrait;
        assert!(matches!(
            Column::ParentId.def().get_column_type(),
            runique::sea_orm::ColumnType::Uuid
        ));
    }
}
