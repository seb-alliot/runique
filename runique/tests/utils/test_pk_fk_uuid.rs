// Regression test: `Pk` on a `belongs_to` column under `pk-uuid`.
//
// `Pk` resolves to `FormFieldKind::Uuid` under this feature. The FK
// declaration (then `fk()`) used to only accept `Int | Bigint`, rejecting
// this at compile time — found 2026-09-02; demo-app runs under `postgres`
// only, so it never exercised `pk-uuid`.
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
            parent_id: Pk [required],
        },
        relations: {
            belongs_to: scratch_parent_uuid_check via parent_id [cascade],
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
