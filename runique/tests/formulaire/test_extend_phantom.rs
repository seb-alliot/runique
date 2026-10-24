//! What `extend!{}` generates for the framework's own columns (the phantom
//! columns of the extended table) and for an enum it declares.
use crate::helpers::pk::pk;
use runique::forms::Forms;
use runique::forms::field::FormAccess;
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
    groups::EihwazGroupesModelForm::register_fields(&mut form);
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
    users::EihwazUsersModelForm::register_fields(&mut form);
    assert!(!form.fields.contains_key("activated_at"));
}

// A profile form never sets the account's password: a new account gets one
// nobody knows (its owner picks theirs from the email link), an edit keeps the
// stored hash — whatever the request carries.
#[test]
fn an_extended_user_never_takes_a_password_from_the_form() {
    let mut data = StrMap::new();
    data.insert("password".into(), "chosen-by-the-admin".into());

    let created = users::admin_from_form(&data, None).expect("converts");
    let ActiveValue::Set(hash) = &created.password else {
        panic!("a new account needs a password");
    };
    assert_ne!(hash, "chosen-by-the-admin");
    assert!(
        hash.starts_with("$argon2"),
        "a hash, not the typed text: {hash}"
    );

    let other = users::admin_from_form(&data, None).expect("converts");
    assert_ne!(
        created.password, other.password,
        "random, not derived from the input"
    );

    let updated = users::admin_partial_update(&data, pk(1)).expect("converts");
    assert_eq!(updated.password, ActiveValue::NotSet);

    let mut form = Forms::new("csrf");
    users::EihwazUsersModelForm::register_fields(&mut form);
    assert!(!form.fields.contains_key("password"));
}

// Activating or deactivating an account goes through the gate's account
// rules, never a profile form: a new account starts inactive (database
// default), an edit leaves the state alone.
#[test]
fn an_extended_user_never_takes_is_active_from_the_form() {
    let mut data = StrMap::new();
    data.insert("is_active".into(), "true".into());

    let created = users::admin_from_form(&data, None).expect("converts");
    assert_eq!(created.is_active, ActiveValue::NotSet);
    let mut data = StrMap::new();
    data.insert("is_active".into(), "false".into());
    let updated = users::admin_partial_update(&data, pk(1)).expect("converts");
    assert_eq!(updated.is_active, ActiveValue::NotSet);

    let mut form = Forms::new("csrf");
    users::EihwazUsersModelForm::register_fields(&mut form);
    assert!(!form.fields.contains_key("is_active"));
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

// An extension of a framework table carries the base table's rules: its own
// columns are added, never a weaker key or protection.
#[cfg(feature = "admin")]
#[test]
fn an_extended_account_table_keeps_the_accounts_rules() {
    use runique::auth::meta::ModelMeta;
    use runique::sea_orm::IdenStatic;
    assert_eq!(<users::Entity as ModelMeta>::KEY, "users");
    const { assert!(<users::Entity as ModelMeta>::IS_ACCOUNT_TABLE) };
    const { assert!(<users::Entity as ModelMeta>::IS_FRAMEWORK_TABLE) };
    assert!(
        <users::Entity as ModelMeta>::SECRET
            .iter()
            .any(|c| c.as_str() == "password")
    );

    assert_eq!(<groups::Entity as ModelMeta>::KEY, "groupes");
    assert_eq!(
        <users::Entity as ModelMeta>::KEY,
        runique::utils::constante::resource_key::USERS
    );
    assert_eq!(
        <groups::Entity as ModelMeta>::KEY,
        runique::utils::constante::resource_key::GROUPES
    );
    assert_eq!(
        <groups::Entity as ModelMeta>::KEY,
        runique::utils::constante::resource_key::GROUPES
    );

    const { assert!(<groups::Entity as ModelMeta>::IS_FRAMEWORK_TABLE) };
    const { assert!(!<groups::Entity as ModelMeta>::IS_ACCOUNT_TABLE) };
}

#[cfg(feature = "admin")]
#[test]
fn an_extended_account_table_replaces_the_accounts_admin() {
    use runique::admin::builder::admin_builder::ModelAdmin;
    use runique::app::staging::AdminStaging;
    let staging = AdminStaging::new()
        .enable()
        .path("/admin")
        .site(|s| s.accounts_users(ModelAdmin::<users::Entity>::new()));
    if let Err(e) = staging.validate() {
        panic!("{e}");
    }

    let wrong = AdminStaging::new()
        .enable()
        .path("/admin")
        .site(|s| s.accounts_users(ModelAdmin::<groups::Entity>::new()));
    assert!(
        wrong.validate().is_err(),
        "the groups can't stand in for the accounts"
    );
}
