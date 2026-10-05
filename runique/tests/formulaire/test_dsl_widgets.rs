//! A field declared in `model!{}` gets the same form field, with the same
//! limits, whether the form is the admin's or a `#[form]` built from the
//! schema — both read `runique_dsl::types`.
//! Regressions covered: `#[form]` guessed the field from the column's name
//! (`contact: email` was plain text), accepted any `i64` for an `i16`, lost
//! enum choices outside Postgres, ignored `[label]` and `[min]`; the admin
//! ignored `[required]` on numbers; `3,5` validated as 3.5 but was saved as is.
use runique::forms::Forms;
use runique::forms::field::RuniqueForm;

mod shop {
    use runique::prelude::*;

    model! {
        Item,
        table: "items",
        pk: id => i32,
        enums: {
            Status: [Draft: "Brouillon", Published],
        },
        {
            contact: email [nullable],
            qty: i16 [required, min: 0, label: "Quantité"],
            ratio: float [nullable],
            price: decimal [nullable],
            status: choice [nullable, enum(Status)],
            photo: image [nullable, upload_to: "photos/"],
        }
    }
}

/// The form a `#[form(schema = shop)]` gets.
fn schema_form() -> Forms {
    let mut form = Forms::new("csrf");
    shop::schema().fill_form(&mut form, None, None);
    form
}

/// The form the admin gets.
fn admin_form() -> Forms {
    let mut form = Forms::new("csrf");
    shop::ItemAdminForm::register_fields(&mut form);
    form
}

async fn check(form: &mut Forms, name: &str, value: &str) -> (bool, String) {
    let field = form.fields.get_mut(name).expect("field registered");
    field.set_value(value);
    let ok = field.validate().await;
    (ok, field.value().to_string())
}

#[test]
fn schema_form_follows_the_declared_type_not_the_name() {
    let form = schema_form();
    assert_eq!(form.fields["contact"].field_type(), "email");
    assert_eq!(form.fields["photo"].field_type(), "file");
    assert_eq!(form.fields["status"].field_type(), "select");
    assert_eq!(form.fields["qty"].label(), "Quantité");
}

#[tokio::test]
async fn both_forms_hold_an_enum_to_its_variants() {
    for mut form in [schema_form(), admin_form()] {
        assert!(check(&mut form, "status", "Draft").await.0);
        assert!(!check(&mut form, "status", "Deleted").await.0);
    }
}

#[tokio::test]
async fn both_forms_hold_an_i16_to_its_range_and_declared_min() {
    for mut form in [schema_form(), admin_form()] {
        assert!(
            !check(&mut form, "qty", "40000").await.0,
            "40000 doesn't fit an i16"
        );
        assert!(!check(&mut form, "qty", "-1").await.0, "[min: 0]");
        assert_eq!(
            check(&mut form, "qty", " 42 ").await,
            (true, "42".to_string())
        );
    }
}

#[tokio::test]
async fn both_forms_require_a_required_number() {
    for mut form in [schema_form(), admin_form()] {
        assert!(!check(&mut form, "qty", "").await.0);
    }
}

#[tokio::test]
async fn a_decimal_comma_is_saved_the_way_it_was_checked() {
    for mut form in [schema_form(), admin_form()] {
        assert_eq!(
            check(&mut form, "ratio", "3,5").await,
            (true, "3.5".to_string())
        );
    }
}

#[tokio::test]
async fn a_decimal_is_checked_as_a_decimal() {
    for mut form in [schema_form(), admin_form()] {
        assert!(
            !check(&mut form, "price", "1e5").await.0,
            "Decimal has no exponent"
        );
        assert_eq!(
            check(&mut form, "price", "12,50").await,
            (true, "12.50".to_string())
        );
    }
}

/// The options a `<select>` gets, as `value=label;…`.
fn rendered_choices(form: &Forms, name: &str) -> String {
    let mut tera = tera::Tera::default();
    tera.add_raw_template(
        "base_select.html",
        "{% for c in choices %}{{ c.value }}={{ c.label }};{% endfor %}",
    )
    .unwrap();
    form.fields[name]
        .render(&std::sync::Arc::new(tera))
        .expect("renders")
}

#[test]
fn both_forms_show_the_enum_labels_declared_in_the_dsl() {
    for form in [schema_form(), admin_form()] {
        assert_eq!(
            rendered_choices(&form, "status"),
            "Draft=Brouillon;Published=Published;"
        );
    }
}
