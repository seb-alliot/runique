//! A checkbox only accepts what it sends: its own value (checked) or nothing
//! (unchecked, which `fill()` turns into "false"). A forged value
//! (`is_active=banana`) is a form error, never read as `false` — it would
//! otherwise deactivate an account without anyone having sent `false`.
//! Below the field (a column, the generated write) a boolean reads six
//! spellings and nothing else.
use runique::forms::FormDataError;
use runique::forms::base::FormField;
use runique::forms::column_value::column_value;
use runique::forms::field::{FormAccess, RuniqueForm};
use runique::forms::fields::BooleanField;
use runique::forms::form::Forms;
use runique::sea_orm::ActiveValue;
use runique::sea_orm::Value;
use runique::sea_orm::sea_query::ColumnType;
use runique::utils::aliases::StrMap;

mod flags {
    use runique::prelude::*;

    model! {
        Flag,
        table: "bool_strict_flags",
        pk: id => i32,
        {
            active: bool,
        }
    }
}

const TRUE: [&str; 4] = ["true", "1", "on", "ON"];
const FALSE: [&str; 3] = ["false", "0", "off"];
const FORGED: [&str; 4] = ["banana", "yes", "2", "truex"];

// ── the one reading ──────────────────────────────────────────────

#[test]
fn column_value_reads_the_six_spellings() {
    for raw in TRUE {
        assert_eq!(
            column_value(&ColumnType::Boolean, raw),
            Ok(Value::from(true)),
            "{raw}"
        );
    }
    for raw in FALSE {
        assert_eq!(
            column_value(&ColumnType::Boolean, raw),
            Ok(Value::from(false)),
            "{raw}"
        );
    }
}

#[test]
fn column_value_refuses_a_forged_boolean() {
    for raw in FORGED {
        assert!(
            column_value(&ColumnType::Boolean, raw).is_err(),
            "`{raw}` read as a boolean"
        );
    }
}

// ── the field: only what the box sends ───────────────────────────

#[tokio::test]
async fn the_box_accepts_its_value_or_unchecked() {
    for raw in ["true", "false", ""] {
        let mut field = BooleanField::new("active");
        field.set_value(raw);
        assert!(field.validate().await, "`{raw}` refused");
    }
}

#[tokio::test]
async fn the_box_refuses_what_it_never_sends() {
    for raw in ["banana", "on", "1", "yes", "TRUE"] {
        let mut field = BooleanField::new("active");
        field.set_value(raw);
        assert!(!field.validate().await, "`{raw}` passed validation");
        assert!(field.to_value().is_err(), "`{raw}` converted to a value");
    }
}

#[tokio::test]
async fn a_box_with_its_own_value_accepts_that_one_only() {
    let mut field = BooleanField::new("cgu").checked_value("on");
    field.set_value("on");
    assert!(field.validate().await);
    assert_eq!(field.to_value(), Ok(Some(Value::from(true))));
    field.set_value("true");
    assert!(!field.validate().await, "not what this box sends");
}

// ── the whole form ───────────────────────────────────────────────

struct FlagForm {
    form: Forms,
}

impl FormAccess for FlagForm {
    fn register_fields(form: &mut Forms) {
        form.field(&BooleanField::new("active"));
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

impl RuniqueForm for FlagForm {}

fn flag_form(active: &str) -> FlagForm {
    let mut form = FlagForm {
        form: Forms::new("csrf"),
    };
    FlagForm::register_fields(&mut form.form);
    form.form.add_value("active", active);
    form
}

#[tokio::test]
async fn a_form_with_a_real_boolean_is_valid() {
    assert!(flag_form("true").is_valid().await);
    assert!(flag_form("false").is_valid().await);
}

#[tokio::test]
async fn a_form_with_a_forged_boolean_is_invalid() {
    let mut form = flag_form("banana");
    assert!(
        !form.is_valid().await,
        "a forged boolean made the form valid"
    );
    assert!(form.get_form().fields["active"].error().is_some());
}

// ── the generated write (`admin_from_form`) ──────────────────────

fn data(active: &str) -> StrMap {
    StrMap::from([("active".to_string(), active.to_string())])
}

#[test]
fn the_generated_write_reads_a_real_boolean() {
    let model = flags::admin_from_form(&data("on"), None).unwrap();
    assert_eq!(model.active, ActiveValue::Set(true));
    let model = flags::admin_from_form(&data("off"), None).unwrap();
    assert_eq!(model.active, ActiveValue::Set(false));
}

#[test]
fn the_generated_write_refuses_a_forged_boolean() {
    let err = flags::admin_from_form(&data("banana"), None).unwrap_err();
    assert_eq!(err, FormDataError::Invalid("active".into()));
}

// ── from the schema: NOT NULL alone doesn't make a box required ──

#[test]
fn a_not_null_bool_column_gives_an_optional_box() {
    use runique::migration::ColumnDef;
    use runique::runique_dsl::ast::FormFieldKind;
    let field = ColumnDef::new("actif")
        .boolean()
        .kind(FormFieldKind::Bool)
        .to_form_field()
        .expect("a box");
    assert!(!field.required(), "NOT NULL alone isn't `required`");
}

#[test]
fn a_declared_required_bool_gives_a_box_to_check() {
    use runique::migration::ColumnDef;
    use runique::runique_dsl::ast::FormFieldKind;
    let field = ColumnDef::new("cgu")
        .boolean()
        .required()
        .kind(FormFieldKind::Bool)
        .to_form_field()
        .expect("a box");
    assert!(field.required());
}

#[test]
fn a_not_null_text_column_stays_required() {
    use runique::migration::ColumnDef;
    use runique::runique_dsl::ast::FormFieldKind;
    let field = ColumnDef::new("titre")
        .kind(FormFieldKind::Text)
        .to_form_field()
        .expect("a text field");
    assert!(field.required(), "only boxes are exempt");
}
