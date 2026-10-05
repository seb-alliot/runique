//! `belongs_to` is the only FK declaration: the macro's relation and schema
//! reference the target entity's real table and primary key, with the
//! declared ON DELETE / ON UPDATE actions.
mod shelf {
    use runique::prelude::*;

    model! {
        Shelf,
        table: "library_shelves",
        pk: code => i32,
        {
            label: text [required],
        }
    }
}

mod book {
    use runique::prelude::*;

    model! {
        Book,
        table: "library_books",
        pk: id => i32,
        {
            title: text [required],
            shelf_code: int [nullable],
        },
        relations: {
            belongs_to: shelf via shelf_code [set_null, cascade],
        }
    }
}

use runique::sea_orm::RelationTrait;
use runique::sea_orm::sea_query::ForeignKeyAction;

#[test]
fn relation_targets_the_primary_key_with_its_actions() {
    let rel = book::Relation::Shelf.def();
    let to: Vec<String> = rel.to_col.iter().map(|c| c.to_string()).collect();
    assert_eq!(to, ["code"]);
    assert!(matches!(rel.on_delete, Some(ForeignKeyAction::SetNull)));
    assert!(matches!(rel.on_update, Some(ForeignKeyAction::Cascade)));
}

#[test]
fn schema_fk_uses_the_target_table_and_primary_key() {
    let schema = book::schema();
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.from_column, "shelf_code");
    assert_eq!(fk.to_table, "library_shelves");
    assert_eq!(fk.to_column, "code");
    assert!(matches!(fk.on_delete, ForeignKeyAction::SetNull));
    assert!(matches!(fk.on_update, ForeignKeyAction::Cascade));
}
