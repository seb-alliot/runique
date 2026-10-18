//! A field renders its cleaned value as its column stores it (`to_value`),
//! is held to that column once given it (`apply_column_type`), and a form
//! whose value can't be stored isn't valid.
use runique::forms::base::FormField;
use runique::forms::column_value::column_value;
use runique::forms::field::RuniqueForm;
use runique::forms::fields::{BooleanField, CheckboxField, HiddenField, NumericField, TextField};
use runique::forms::form::Forms;
use runique::forms::generic::{GenericField, accepts};
use runique::sea_orm::Value;
use runique::sea_orm::prelude::Decimal;
use runique::sea_orm::sea_query::{ColumnType, StringLen};

// ── column_value: the one reading of a field's text ─────────────

#[test]
fn an_integer_is_read_at_its_column_width() {
    assert_eq!(
        column_value(&ColumnType::SmallInteger, "300"),
        Ok(Value::from(300i16))
    );
    assert!(
        column_value(&ColumnType::SmallInteger, "40000").is_err(),
        "beyond an i16"
    );
    assert!(column_value(&ColumnType::Integer, "12a").is_err());
}

#[test]
fn a_decimal_is_read_exactly_with_either_separator() {
    assert_eq!(
        column_value(&ColumnType::Decimal(Some((10, 2))), "9,99"),
        Ok(Value::from(Decimal::new(999, 2)))
    );
    assert!(
        column_value(&ColumnType::Decimal(None), "1e5").is_err(),
        "no exponent"
    );
}

#[test]
fn a_boolean_is_true_only_when_checked() {
    assert_eq!(
        column_value(&ColumnType::Boolean, "on"),
        Ok(Value::from(true))
    );
    assert_eq!(
        column_value(&ColumnType::Boolean, "false"),
        Ok(Value::from(false))
    );
}

#[test]
fn malformed_text_for_a_typed_column_is_refused() {
    assert!(column_value(&ColumnType::Uuid, "not-a-uuid").is_err());
    assert!(column_value(&ColumnType::Json, "{oops").is_err());
    assert!(column_value(&ColumnType::Date, "31/12/2026").is_err());
    assert!(
        column_value(&ColumnType::Blob, "%%%").is_err(),
        "not base64"
    );
    assert!(column_value(&ColumnType::Uuid, "67e55044-10b1-426f-9247-bb680e5fe0c8").is_ok());
}

#[test]
fn a_column_no_field_fills_is_refused() {
    assert!(column_value(&ColumnType::Inet, "127.0.0.1").is_err());
}

// ── to_value ─────────────────────────────────────────────────────

#[test]
fn an_empty_field_has_no_value() {
    let field = TextField::text("title");
    assert_eq!(field.to_value(), Ok(None));
}

#[test]
fn without_a_column_a_field_uses_its_own_type() {
    let mut field = NumericField::integer("qty");
    field.set_value("12");
    assert_eq!(field.to_value(), Ok(Some(Value::from(12i64))));
}

#[test]
fn given_a_column_a_field_renders_its_exact_type() {
    let mut field = NumericField::integer("qty");
    field.apply_column_type(&ColumnType::SmallInteger);
    field.set_value("12");
    assert_eq!(field.to_value(), Ok(Some(Value::from(12i16))));
}

#[test]
fn an_unchecked_box_is_false_not_empty() {
    let field = BooleanField::new("active");
    assert_eq!(field.to_value(), Ok(Some(Value::from(false))));
}

#[test]
fn a_list_and_an_unbound_hidden_field_fill_no_column() {
    let mut list = CheckboxField::new("tags");
    list.set_value("a,b");
    assert_eq!(list.to_value(), Ok(None));
    let mut hidden = HiddenField::new("next");
    hidden.set_value("/home");
    assert_eq!(hidden.to_value(), Ok(None));
    hidden.apply_column_type(&ColumnType::Text);
    assert_eq!(
        hidden.to_value(),
        Ok(Some(Value::from("/home".to_string())))
    );
}

// ── a password never leaves in clear ─────────────────────────────

#[tokio::test]
async fn a_password_leaves_only_once_hashed() {
    let mut field = TextField::password("password");
    field.set_value("S3cret-passw0rd!");
    assert_eq!(field.to_value(), Ok(None), "typed in, not hashed yet");

    field.finalize().await.expect("hashed");
    let Ok(Some(Value::String(Some(stored)))) = field.to_value() else {
        panic!("a hashed password is stored");
    };
    assert_ne!(stored, "S3cret-passw0rd!");
}

#[tokio::test]
async fn a_no_hash_password_never_leaves() {
    let mut field = TextField::password("password").no_hash();
    field.set_value("S3cret-passw0rd!");
    field.finalize().await.expect("nothing to do");
    assert_eq!(field.to_value(), Ok(None));
}

// ── apply_column_type: held to the column, never loosened ────────

#[tokio::test]
async fn an_integer_column_bounds_the_field_to_its_width() {
    let mut field = NumericField::integer("qty");
    field.apply_column_type(&ColumnType::SmallInteger);
    field.set_value("32767");
    assert!(field.validate().await, "the largest i16");
    field.set_value("32768");
    assert!(!field.validate().await);
}

#[tokio::test]
async fn a_decimal_column_sets_the_precision_but_keeps_a_tighter_one() {
    let mut field = NumericField::decimal("price");
    field.apply_column_type(&ColumnType::Decimal(Some((6, 2))));
    field.set_value("9999.99");
    assert!(field.validate().await);
    field.set_value("1.999");
    assert!(!field.validate().await);

    let mut tighter = NumericField::decimal("price").decimal_len(4, 1);
    tighter.apply_column_type(&ColumnType::Decimal(Some((6, 2))));
    tighter.set_value("9.99");
    assert!(!tighter.validate().await, "1 place, not the column's 2");
}

#[tokio::test]
async fn a_varchar_bounds_a_text_field_but_keeps_a_tighter_one() {
    let mut field = TextField::text("code");
    field.apply_column_type(&ColumnType::String(StringLen::N(5)));
    field.set_value("12345");
    assert!(field.validate().await);
    field.set_value("123456");
    assert!(!field.validate().await);

    let mut tighter = TextField::text("code").max_length(3, "");
    tighter.apply_column_type(&ColumnType::String(StringLen::N(5)));
    tighter.set_value("1234");
    assert!(!tighter.validate().await, "3, not the column's 5");
}

// ── accepts: the right kind of field for the column ──────────────

#[test]
fn a_field_stores_only_its_kind_of_column() {
    let decimal = ColumnType::Decimal(Some((10, 2)));
    let text: GenericField = TextField::text("t").into();
    let price: GenericField = NumericField::decimal("p").into();
    let count: GenericField = NumericField::integer("c").into();
    let flag: GenericField = BooleanField::new("f").into();
    let list: GenericField = CheckboxField::new("l").into();
    assert!(!accepts(&text.field, &decimal), "free text in a price");
    assert!(accepts(&price.field, &decimal));
    assert!(
        !accepts(&count.field, &decimal),
        "an integer field in a price"
    );
    assert!(accepts(&count.field, &ColumnType::SmallInteger));
    assert!(!accepts(&flag.field, &ColumnType::Integer));
    assert!(accepts(&flag.field, &ColumnType::Boolean));
    assert!(
        !accepts(&list.field, &ColumnType::Text),
        "a list has no column"
    );
}

// ── a value the column can't store makes the form invalid ────────

struct StockForm {
    form: Forms,
}

impl RuniqueForm for StockForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("qty"));
    }
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

fn stock_form(qty: &str) -> StockForm {
    let mut form = StockForm {
        form: Forms::new("csrf"),
    };
    StockForm::register_fields(&mut form.form);
    // The column the field is stored in, as the admin gives it.
    form.form
        .fields
        .get_mut("qty")
        .expect("qty")
        .apply_column_type(&ColumnType::SmallInteger);
    form.form.add_value("qty", qty);
    form
}

#[tokio::test]
async fn a_value_the_column_can_store_keeps_the_form_valid() {
    let mut form = stock_form("12");
    assert!(form.is_valid().await);
}

#[tokio::test]
async fn a_value_the_column_cant_store_is_a_field_error() {
    let mut form = stock_form("douze");
    assert!(!form.is_valid().await);
    assert!(
        form.get_form().fields["qty"].error().is_some(),
        "the error is on the field, shown under it"
    );
}

// ── cleaned::<T>: the developer's typed reading ──────────────────

fn qty_form(qty: &str) -> StockForm {
    let mut form = StockForm {
        form: Forms::new("csrf"),
    };
    StockForm::register_fields(&mut form.form);
    form.form.add_value("qty", qty);
    form
}

#[test]
fn cleaned_reads_the_text_at_the_width_asked_for() {
    let form = qty_form("40000");
    assert_eq!(form.cleaned::<i16>("qty"), None, "beyond an i16");
    assert_eq!(form.cleaned::<i32>("qty"), Some(40000));
}

#[test]
fn cleaned_reads_a_decimal_exactly() {
    let form = qty_form("9,99");
    assert_eq!(form.cleaned::<Decimal>("qty"), Some(Decimal::new(999, 2)));
    assert_eq!(form.cleaned::<i32>("qty"), None, "not an integer");
}

#[test]
fn cleaned_only_reads_a_declared_field() {
    let form = qty_form("12");
    assert_eq!(form.cleaned::<i32>("unknown"), None);
}
