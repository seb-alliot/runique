//! A field declared `password` in `model!{}` stays a password field once a
//! `#[form]` rebuilds it from the schema, whatever the column is called.
//! Regression: the form used to guess from the name (`password`, `*_password`,
//! `*_pwd`), so `secret: password` became a plain text field — value accepted
//! from a GET, echoed back to templates, never hashed.
use axum::http::Method;
use runique::forms::Forms;
use runique::utils::aliases::StrMap;

mod account {
    use runique::prelude::*;

    model! {
        Account,
        table: "accounts",
        pk: id => i32,
        {
            secret: password [required],
        }
    }
}

fn secret_form() -> Forms {
    let schema = account::schema();
    let column = schema
        .columns
        .iter()
        .find(|c| c.name == "secret")
        .expect("secret column");
    assert_eq!(
        column.kind,
        Some(runique::runique_dsl::ast::FormFieldKind::Password),
        "schema() must carry the DSL type"
    );

    let mut form = Forms::new("csrf");
    form.field_generic(column.to_form_field().expect("form field"));
    form
}

fn data(value: &str) -> StrMap {
    let mut data = StrMap::new();
    data.insert("secret".to_string(), value.to_string());
    data
}

#[test]
fn password_column_builds_a_password_field() {
    let form = secret_form();
    let field = form.fields.get("secret").expect("field registered");
    assert_eq!(field.field_type(), "password");
    assert!(field.is_password());
}

#[test]
fn password_value_is_ignored_on_get() {
    let mut form = secret_form();
    form.fill(&data("hunter2"), Method::GET);
    assert_eq!(form.fields["secret"].value(), "");
}

#[test]
fn password_value_never_reaches_the_templates() {
    let mut form = secret_form();
    form.fill(&data("hunter2"), Method::POST);
    let json = serde_json::to_value(&form).expect("serializes");
    assert_eq!(json["fields"]["secret"]["value"], "");
}

#[tokio::test]
async fn password_value_is_hashed_on_finalize() {
    let mut form = secret_form();
    form.fill(&data("hunter2"), Method::POST);
    form.finalize().await.expect("finalize");
    let stored = form.fields["secret"].value();
    assert!(stored.starts_with("$argon2"), "stored {stored:?}");
}
