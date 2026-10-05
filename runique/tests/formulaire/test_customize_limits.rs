//! `customize` can adjust a field built from the schema, not loosen what the
//! column declares: an integer keeps its type's range, a declared
//! `min_length`/`max_length`/`min`/`max` can be tightened but loosening one
//! panics, and a replacement that breaks what the column needs is refused.
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
            code: text [nullable, min_length: 2, max_length: 10],
            level: int [required, min: 0, max: 100],
            price: float [required, min: 0.5, max: 50.0],
            secret: password [nullable],
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

schema_form!(WiderIntegerType, |form| {
    // An i64-range field in place of the schema's i16.
    form.field(&NumericField::integer("qty").label("Quantity"));
});

schema_form!(Tightened, |form| {
    form.field(&TextField::text("code").min_length(3, "").max_length(8, ""));
    form.field(&NumericField::integer("level").min(10.0, "").max(90.0, ""));
    form.field(&NumericField::float("price").min(1.0, "").max(20.0, ""));
});

schema_form!(LongerText, |form| {
    form.field(
        &TextField::text("code")
            .min_length(2, "")
            .max_length(500, ""),
    );
});

schema_form!(ShorterMinimum, |form| {
    form.field(&TextField::text("code").min_length(1, "").max_length(10, ""));
});

schema_form!(NoTextLimit, |form| {
    form.field(&TextField::text("code"));
});

schema_form!(HigherIntegerMax, |form| {
    form.field(&NumericField::integer("level").min(0.0, "").max(1000.0, ""));
});

schema_form!(LowerIntegerMin, |form| {
    form.field(&NumericField::integer("level").min(-5.0, "").max(100.0, ""));
});

schema_form!(HigherFloatMax, |form| {
    form.field(&NumericField::float("price").min(0.5, "").max(99.0, ""));
});

schema_form!(LowerFloatMin, |form| {
    form.field(&NumericField::float("price").min(0.0, "").max(50.0, ""));
});

schema_form!(PasswordAsText, |form| {
    form.field(&TextField::text("secret"));
});

schema_form!(PasswordAsSecretText, |form| {
    let mut secret = TextField::text("secret");
    secret.base.mark_password();
    form.field(&secret);
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
async fn an_integer_keeps_its_type_range() {
    let mut form = Forms::new("csrf");
    WiderIntegerType::register_fields(&mut form);
    assert_eq!(
        form.fields["qty"].label(),
        "Quantity",
        "the adjustment is kept"
    );
    assert!(!check(&mut form, "qty", "40000").await, "still an i16");
}

#[tokio::test]
async fn customize_may_tighten_the_declared_bounds() {
    let mut form = Forms::new("csrf");
    Tightened::register_fields(&mut form);
    assert!(
        !check(&mut form, "code", "123456789").await,
        "8 at most now"
    );
    assert!(check(&mut form, "code", "12345678").await);
    assert!(!check(&mut form, "level", "95").await, "90 at most now");
    assert!(!check(&mut form, "price", "0.7").await, "1.0 at least now");
}

fn register<F: RuniqueForm>() {
    let mut form = Forms::new("csrf");
    F::register_fields(&mut form);
}

#[test]
#[should_panic(expected = "loosened `max_length`")]
fn a_longer_max_length_panics() {
    register::<LongerText>();
}

#[test]
#[should_panic(expected = "loosened `max_length`")]
fn dropping_the_max_length_panics() {
    register::<NoTextLimit>();
}

#[test]
#[should_panic(expected = "loosened `min_length`")]
fn a_shorter_min_length_panics() {
    register::<ShorterMinimum>();
}

#[test]
#[should_panic(expected = "loosened `max`")]
fn a_higher_integer_max_panics() {
    register::<HigherIntegerMax>();
}

#[test]
#[should_panic(expected = "loosened `min`")]
fn a_lower_integer_min_panics() {
    register::<LowerIntegerMin>();
}

#[test]
#[should_panic(expected = "loosened `max`")]
fn a_higher_float_max_panics() {
    register::<HigherFloatMax>();
}

#[test]
#[should_panic(expected = "loosened `min`")]
fn a_lower_float_min_panics() {
    register::<LowerFloatMin>();
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

// Marked secret is not enough: a `password` column needs the password field
// itself (hashed), not a text field that only hides its value.
#[test]
#[should_panic(expected = "is declared `password`")]
fn customize_cannot_turn_a_password_into_secret_text() {
    let mut form = Forms::new("csrf");
    PasswordAsSecretText::register_fields(&mut form);
}
