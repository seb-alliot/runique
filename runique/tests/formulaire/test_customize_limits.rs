//! `customize` can adjust a field built from the schema, not loosen what the
//! column's DSL type imposes: those limits are applied again after it runs,
//! and a replacement that breaks what the column needs is refused.
use runique::forms::Forms;
use runique::forms::field::RuniqueForm;
use runique::forms::fields::{NumericField, TextField};

mod stock {
    use runique::prelude::*;

    model! {
        Stock,
        table: "stocks",
        pk: id => i32,
        {
            qty: i16 [required],
            code: text [max_length: 10],
            secret: password,
        }
    }
}

macro_rules! schema_form {
    ($name:ident, |$form:ident| $body:block) => {
        #[runique::prelude::form(schema = stock)]
        pub struct $name;

        #[runique::prelude::async_trait]
        impl RuniqueForm for $name {
            runique::impl_form_access!(model);

            fn customize($form: &mut Forms) $body
        }
    };
}

schema_form!(Loosened, |form| {
    // An i64-range field and a longer text, in place of the schema's.
    form.field(&NumericField::integer("qty").label("Quantity"));
    form.field(&TextField::text("code").max_length(500, ""));
});

schema_form!(PasswordAsText, |form| {
    form.field(&TextField::text("secret"));
});

schema_form!(IntegerAsText, |form| {
    form.field(&TextField::text("qty"));
});

async fn check(form: &mut Forms, name: &str, value: &str) -> bool {
    let field = form.fields.get_mut(name).expect("field");
    field.set_value(value);
    field.validate().await
}

#[tokio::test]
async fn customize_cannot_loosen_the_column_limits() {
    let mut form = Forms::new("csrf");
    Loosened::register_fields(&mut form);
    assert_eq!(
        form.fields["qty"].label(),
        "Quantity",
        "the adjustment is kept"
    );
    assert!(!check(&mut form, "qty", "40000").await, "still an i16");
    assert!(
        !check(&mut form, "code", "12345678901").await,
        "still 10 characters"
    );
    assert!(check(&mut form, "code", "1234567890").await);
}

#[test]
#[should_panic(expected = "is declared `password`")]
fn customize_cannot_turn_a_password_into_plain_text() {
    let mut form = Forms::new("csrf");
    PasswordAsText::register_fields(&mut form);
}

#[test]
#[should_panic(expected = "must stay an integer field")]
fn customize_cannot_turn_an_integer_into_text() {
    let mut form = Forms::new("csrf");
    IntegerAsText::register_fields(&mut form);
}
