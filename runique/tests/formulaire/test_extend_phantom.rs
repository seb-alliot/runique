//! What `extend!{}` generates for the framework's own columns (the phantom
//! columns of the extended table) and for an enum it declares.
use crate::helpers::pk::pk;
use runique::forms::Forms;
use runique::forms::field::RuniqueForm;
use runique::sea_orm::{ActiveValue, PrimaryKeyTrait};
use runique::utils::aliases::StrMap;

mod groups {
    use runique::prelude::*;

    extend! {
        table: "eihwaz_groupes",
        enums: {
            Tier: [Bronze, Gold = ("gold", "Or")],
        },
        fields: {
            tier: choice [enum(Tier), nullable],
        }
    }
}

#[test]
fn a_framework_column_is_a_field_of_the_admin_form_with_its_label() {
    let mut form = Forms::new("csrf");
    groups::EihwazGroupesAdminForm::register_fields(&mut form);
    let field = form.fields.get("nom").expect("nom registered");
    assert_eq!(field.label(), "Nom");
}

#[test]
fn a_framework_column_is_written_on_create_and_on_partial_update() {
    let mut data = StrMap::new();
    data.insert("nom".into(), "  Editors ".into());

    let created = groups::admin_from_form(&data, None).expect("converts");
    assert_eq!(created.nom, ActiveValue::Set("Editors".to_string()));

    let updated = groups::admin_partial_update(&data, pk(1)).expect("converts");
    assert_eq!(updated.nom, ActiveValue::Set("Editors".to_string()));

    let untouched = groups::admin_partial_update(&StrMap::new(), pk(1)).expect("converts");
    assert_eq!(untouched.nom, ActiveValue::NotSet);
}

#[test]
fn a_declared_enum_is_shown_with_its_label_in_the_admin() {
    let mut row = runique::serde_json::json!({ "tier": "gold", "nom": "gold" });
    groups::apply_enum_labels(&mut row);
    assert_eq!(row["tier"], "Or");
    assert_eq!(row["nom"], "gold", "only the enum column is relabelled");
}

mod users {
    use runique::prelude::*;

    extend! {
        table: "eihwaz_users",
        fields: {
            bio: textarea [nullable],
        }
    }
}

// The database refuses `is_active` without `activated_at`: the extended model
// must read it back, and no form may write it.
#[test]
fn an_extended_user_carries_activated_at_and_no_form_writes_it() {
    let mut data = StrMap::new();
    data.insert("activated_at".into(), "2026-01-01T00:00".into());

    let created = users::admin_from_form(&data, None).expect("converts");
    assert_eq!(created.activated_at, ActiveValue::NotSet);
    let updated = users::admin_partial_update(&data, pk(1)).expect("converts");
    assert_eq!(updated.activated_at, ActiveValue::NotSet);

    let mut form = Forms::new("csrf");
    users::EihwazUsersAdminForm::register_fields(&mut form);
    assert!(!form.fields.contains_key("activated_at"));
}

// `Pk` is an alias: SeaORM can't infer auto-increment through it, the
// generated attribute must say it (none under `pk-uuid`, ids come from the app).
#[test]
fn the_id_of_an_extended_table_follows_the_pk_feature() {
    assert_eq!(
        <groups::PrimaryKey as PrimaryKeyTrait>::auto_increment(),
        !cfg!(feature = "pk-uuid")
    );
}

#[test]
fn the_schema_of_an_extend_holds_its_own_columns() {
    let schema = groups::schema();
    assert_eq!(schema.table_name, "eihwaz_groupes");
    assert!(schema.columns.iter().any(|c| c.name == "tier"));
}
