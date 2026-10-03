//! Form engine guarantees, written from cargo-mutants survivors (2026-10-02):
//! each test fails when the matching check is altered — not only when the
//! form misbehaves today.
use crate::helpers::{request::build_handler_req, server::build_engine};
use axum::http::Method;
use runique::forms::base::{CommonFieldConfig, FormField};
use runique::forms::field::RuniqueForm;
use runique::forms::fields::{
    BooleanField, CheckboxField, ChoiceField, DateField, DateTimeField, DurationField,
    HoneypotField, RadioField, TextField, TimeField,
};
use runique::forms::generic::GenericField;
use runique::forms::{FormRenderer, Forms};
use runique::prelude::ValidationForm;
use runique::utils::aliases::StrMap;
use std::collections::HashMap;
use std::sync::Arc;

fn tera(templates: &[(&str, &str)]) -> Arc<tera::Tera> {
    let mut tera = tera::Tera::default();
    for (name, body) in templates {
        tera.add_raw_template(name, body).unwrap();
    }
    Arc::new(tera)
}

fn data(pairs: &[(&str, &str)]) -> StrMap {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ── A form whose save is observable ─────────────────────────────────────────

struct NoteForm {
    form: Forms,
    saved: bool,
}

#[async_trait::async_trait]
impl RuniqueForm for NoteForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("title").required());
        form.field(&TextField::password("secret"));
    }
    fn from_form(form: Forms) -> Self {
        Self { form, saved: false }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
    async fn on_save(
        &mut self,
        _txn: &runique::sea_orm::DatabaseTransaction,
    ) -> Result<(), runique::sea_orm::DbErr> {
        self.saved = true;
        Ok(())
    }
}

fn note_form() -> NoteForm {
    let mut form = Forms::new("csrf");
    NoteForm::register_fields(&mut form);
    NoteForm::from_form(form)
}

async fn memory_db() -> runique::db::ADb {
    runique::db::ADb::from_connection(
        runique::sea_orm::Database::connect("sqlite::memory:")
            .await
            .unwrap(),
    )
}

#[tokio::test]
async fn nothing_is_saved_before_a_successful_validation() {
    let db = memory_db().await;
    let mut form = note_form();
    assert!(form.save(&db).await.is_err(), "never validated");
    assert!(!form.saved);

    form.get_form_mut()
        .fill(&data(&[("title", "")]), Method::POST);
    assert!(!form.is_valid().await);
    assert!(form.save(&db).await.is_err(), "validated but invalid");
    assert!(!form.saved);

    let mut form = note_form();
    form.get_form_mut()
        .fill(&data(&[("title", "hello")]), Method::POST);
    assert!(form.is_valid().await);
    form.save(&db).await.expect("valid form saves");
    assert!(form.saved, "on_save ran");
}

#[tokio::test]
async fn a_database_error_reaches_the_form() {
    let mut form = note_form();
    form.database_error(&runique::sea_orm::DbErr::Custom("boom".into()));
    assert!(form.get_form().errors.iter().any(|e| e.contains("boom")));
}

// ── fill / add_value ────────────────────────────────────────────────────────

#[test]
fn fill_takes_every_field_but_a_password_on_get() {
    let mut form = note_form();
    form.get_form_mut()
        .fill(&data(&[("title", "hi"), ("secret", "pw")]), Method::GET);
    assert_eq!(form.get_form().fields["title"].value(), "hi");
    assert_eq!(
        form.get_form().fields["secret"].value(),
        "",
        "a password never comes from a GET"
    );

    let mut form = note_form();
    form.get_form_mut()
        .fill(&data(&[("title", "hi"), ("secret", "pw")]), Method::POST);
    assert_eq!(form.get_form().fields["secret"].value(), "pw");
}

#[test]
fn only_a_real_value_marks_the_form_submitted() {
    let mut form = note_form();
    form.get_form_mut().add_value("csrf_token", "tok");
    assert!(
        !form.is_submitted(),
        "the CSRF token alone isn't a submission"
    );
    form.get_form_mut().add_value("title", "  ");
    assert!(!form.is_submitted(), "blank isn't either");
    form.get_form_mut().add_value("title", "hi");
    assert!(form.is_submitted());
}

// ── Database errors attributed to fields ────────────────────────────────────

fn form_with(names: &[&str]) -> Forms {
    let mut form = Forms::new("csrf");
    for name in names {
        form.field(&TextField::text(name));
    }
    form
}

fn custom(msg: &str) -> runique::sea_orm::DbErr {
    runique::sea_orm::DbErr::Custom(msg.into())
}

#[test]
fn a_unique_violation_lands_on_its_field_on_every_engine() {
    for msg in [
        r#"duplicate key value violates unique constraint "users_email_key""#,
        "UNIQUE constraint failed: users.email",
        "Duplicate entry 'a@b.c' for key 'users.email'",
    ] {
        let mut form = form_with(&["email"]);
        form.database_error(&custom(msg));
        assert!(form.fields["email"].error().is_some(), "{msg}");
        assert!(form.errors.is_empty(), "{msg}: not a global error");
    }
}

#[test]
fn the_field_is_the_segment_before_key() {
    let mut form = form_with(&["title", "item"]);
    form.database_error(&custom(
        r#"duplicate key value violates unique constraint "site_page_menu_item_title_key""#,
    ));
    assert!(form.fields["title"].error().is_some());
    assert!(form.fields["item"].error().is_none());
}

#[test]
fn a_foreign_key_constraint_is_not_a_field_error() {
    let mut form = form_with(&["id"]);
    form.database_error(&custom(
        r#"duplicate key value violates unique constraint "orders_user_id_fkey""#,
    ));
    assert!(form.fields["id"].error().is_none());
}

#[test]
fn any_other_database_error_is_global() {
    let mut form = form_with(&["email"]);
    form.database_error(&custom("connection reset"));
    assert!(form.fields["email"].error().is_none());
    assert_eq!(form.errors.len(), 1);
}

// ── Choices, honeypot, scripts ──────────────────────────────────────────────

#[tokio::test]
async fn field_choices_turns_a_field_into_a_select_keeping_value_and_required() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("country").required());
    form.add_value("country", "fr");
    form.field_choices(
        "country",
        "Pays",
        vec![
            ("fr".into(), "France".into()),
            ("de".into(), "Allemagne".into()),
        ],
    );
    let field = form.fields.get_mut("country").unwrap();
    assert_eq!(field.field_type(), "select");
    assert_eq!(field.value(), "fr");
    assert!(field.required());
    field.set_value("xx");
    assert!(!field.validate().await, "only the given choices");
}

#[test]
fn the_honeypot_and_the_scripts_reach_the_template() {
    let mut form = Forms::new("csrf");
    form.set_renderer(FormRenderer::new(tera(&[
        ("base_honeypot.html", "<trap {{ field.name }}>"),
        (
            "js_files.html",
            "{{ csp_nonce }}|{{ js_files | join(sep=',') }}",
        ),
    ])));
    form.set_honeypot("hp_trap");
    form.add_js(&["app.js"]);
    form.set_csp_nonce("N0NCE");
    let json = serde_json::to_value(&form).unwrap();
    assert_eq!(json["honeypot_html"], "<trap hp_trap>");
    assert_eq!(json["rendered_js"], "N0NCE|app.js");
    assert!(!format!("{form:?}").is_empty());
}

// ── ValidationForm ──────────────────────────────────────────────────────────

struct PlainForm {
    form: Forms,
}

impl RuniqueForm for PlainForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("q").required());
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

#[tokio::test]
async fn a_get_is_never_validated_unless_the_form_opts_in() {
    let engine = build_engine().await;
    let mut query = HashMap::new();
    query.insert("q".to_string(), "rust".to_string());
    let request = build_handler_req(engine, None, query, Method::GET).await;
    let form: PlainForm = request.form();
    assert!(
        ValidationForm::try_new(form, &request).await.is_err(),
        "a link must not trigger a form's side effects"
    );
}

#[tokio::test]
async fn validation_form_records_a_database_error() {
    let engine = build_engine().await;
    let mut body = HashMap::new();
    body.insert("q".to_string(), "rust".to_string());
    let request = build_handler_req(engine, None, body, Method::POST).await;
    let form: PlainForm = request.form();
    let Ok(mut validated) = ValidationForm::try_new(form, &request).await else {
        panic!("valid POST");
    };
    validated.database_error(&custom("boom"));
    assert!(
        validated
            .into_form()
            .get_form()
            .errors
            .iter()
            .any(|e| e.contains("boom"))
    );
}

// ── Field config, generic wrapper ───────────────────────────────────────────

#[test]
fn a_field_is_a_password_by_type_or_by_mark() {
    let mut text = TextField::text("api_key");
    assert!(!text.base.is_password());
    text.base.mark_password();
    assert!(text.base.is_password(), "marked");
    assert!(TextField::password("pw").base.is_password(), "by type");
}

#[test]
fn field_defaults_template_and_size() {
    let mut text = TextField::text("t");
    assert_eq!(FormField::template_name(&text), "base_string.html");
    assert_eq!(text.model_max_size(), None);
    assert!(
        text.set_max_size_bounded(runique::forms::fields::FileSize::kb(1))
            .is_err()
    );
}

#[test]
fn the_generic_wrapper_passes_every_setting_on() {
    let mut field: GenericField = TextField::text("t").into();
    assert_eq!(field.template_name(), "base_string.html");
    field.set_placeholder("type here");
    field.set_html_attribute("data-x", "1");
    field.set_readonly(true, None);
    field.set_disabled(true, None);
    let config = field.get_field_config();
    assert_eq!(config.placeholder, "type here");
    assert_eq!(field.to_json_attributes()["data-x"], "1");
    assert!(field.to_json_readonly()["choice"].as_bool().unwrap());
    assert!(field.to_json_disabled()["choice"].as_bool().unwrap());

    let mut bytes: GenericField = runique::forms::fields::BinaryField::new("b")
        .max_size(50)
        .into();
    bytes.cap_max_size(5);
    let shown = serde_json::to_string(&bytes).unwrap();
    assert!(
        shown.contains(r#""max_size":5"#),
        "cap reaches the wrapped field: {shown}"
    );
}

// ── Choice, radio, checkbox ─────────────────────────────────────────────────

#[tokio::test]
async fn an_empty_optional_multiple_select_is_valid() {
    let mut field = ChoiceField::new("tags").multiple().add_choice("a", "A");
    field.set_value("");
    assert!(field.validate().await);
}

#[test]
fn radio_and_checkbox_render_their_choices() {
    let t = tera(&[
        (
            "base_radio.html",
            "{% for c in choices %}{{ c.value }}={{ c.label }};{% endfor %}",
        ),
        (
            "base_checkbox.html",
            "{% for c in choices %}{{ c.value }}:{{ c.selected }};{% endfor %}",
        ),
    ]);
    let radio = RadioField::new("r")
        .add_choice("y", "Yes")
        .add_choice("n", "No");
    assert_eq!(radio.render(&t).unwrap(), "y=Yes;n=No;");
    let mut boxes = CheckboxField::new("c")
        .add_choice("a", "A")
        .add_choice("b", "B");
    boxes.set_value("b");
    assert_eq!(boxes.render(&t).unwrap(), "a:false;b:true;");

    let mut radio = RadioField::new("r").add_choice("y", "Yes").required();
    radio.set_value("y");
    radio.set_html_attribute("data-k", "v");
    assert_eq!(radio.to_json_value(), "y");
    assert_eq!(radio.to_json_required()["choice"], true);
    assert_eq!(radio.to_json_attributes()["data-k"], "v");
}

/// Documented choice: `required` on a checkbox means NOT NULL, not "must be
/// checked" (unlike Django) — a terms box is enforced in `clean()`.
#[tokio::test]
async fn a_checkbox_is_always_valid_and_drops_an_old_error() {
    let mut field = BooleanField::new("terms").required();
    field.set_value("false");
    field.set_error("previous".into());
    assert!(field.validate().await, "unchecked is a valid answer");
    assert!(field.error().is_none(), "the old error is cleared");
}

#[tokio::test]
async fn the_honeypot_never_fails_validation_itself() {
    let mut field = HoneypotField::new("hp");
    field.set_value("bot text");
    assert!(
        field.validate().await,
        "Request::form() rejects a filled trap, not the field"
    );
    let t = tera(&[("base_honeypot.html", "<{{ field.name }}>")]);
    assert_eq!(field.render(&t).unwrap(), "<hp>");
}

// ── Dates: bounds are inclusive ─────────────────────────────────────────────

#[tokio::test]
async fn date_and_time_bounds_are_inclusive() {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    let day = NaiveDate::from_ymd_opt(2026, 10, 2).unwrap();
    let mut date = DateField::new("d").min(day, "").max(day, "");
    date.set_value("2026-10-02");
    assert!(date.validate().await);

    let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let mut time = TimeField::new("t").min(noon, "").max(noon, "");
    time.set_value("12:00");
    assert!(time.validate().await);

    let at = NaiveDateTime::parse_from_str("2026-10-02T12:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut dt = DateTimeField::new("dt").min(at, "").max(at, "");
    dt.set_value("2026-10-02T12:00");
    assert!(dt.validate().await);

    let mut duration = DurationField::new("dur")
        .min_seconds(60, "")
        .max_seconds(60, "");
    duration.set_value("60");
    assert!(duration.validate().await);
}

#[test]
fn date_fields_render_through_their_template() {
    let t = tera(&[("base_datetime.html", "{{ field.name }}={{ field.value }}")]);
    let mut date = DateField::new("d");
    date.set_value("2026-10-02");
    assert_eq!(date.render(&t).unwrap(), "d=2026-10-02");
    let mut time = TimeField::new("t");
    time.set_value("12:00");
    assert_eq!(time.render(&t).unwrap(), "t=12:00");
    let mut duration = DurationField::new("dur");
    duration.set_value("60");
    assert_eq!(duration.render(&t).unwrap(), "dur=60");
    // A fraction of a second on a whole minute keeps the seconds shown.
    let mut dt = DateTimeField::new("dt");
    dt.set_value("2026-10-02T14:30:00.5");
    assert_eq!(dt.render(&t).unwrap(), "dt=2026-10-02T14:30:00");
}

// ── Validator ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn field_errors_and_global_errors_are_told_apart() {
    use runique::forms::validator::ValidationError;
    let mut form = form_with(&["name"]);
    form.fields
        .get_mut("name")
        .unwrap()
        .set_required(true, None);
    assert!(matches!(
        form.is_valid().await,
        Err(ValidationError::FieldValidation(_))
    ));

    let mut form = form_with(&["name"]);
    form.add_value("name", "ok");
    form.errors.push("global".into());
    assert!(matches!(
        form.is_valid().await,
        Err(ValidationError::GlobalErrors(_))
    ));
}

// ── The `form` Tera filter ──────────────────────────────────────────────────

fn render_form(template: &str, form: serde_json::Value) -> String {
    let mut tera = tera::Tera::default();
    tera.register_filter("form", runique::context::tera::form::FormFilter);
    tera.add_raw_template("t", template).unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("f", &form);
    tera.render("t", &ctx).unwrap()
}

fn two_field_form(honeypot: &str) -> serde_json::Value {
    serde_json::json!({
        "fields": {
            "csrf_token": { "name": "csrf_token", "index": 0 },
            "name": { "name": "name", "index": 1 },
            "email": { "name": "email", "index": 2 }
        },
        "rendered_fields": {
            "csrf_token": "<CSRF>",
            "name": "<NAME>",
            "email": "<EMAIL>"
        },
        "honeypot_html": honeypot
    })
}

#[test]
fn the_honeypot_follows_the_last_field_only() {
    let form = two_field_form("<TRAP>");
    assert_eq!(
        render_form(r#"{{ f | form(field="email") }}"#, form.clone()),
        "<EMAIL>\n<TRAP>"
    );
    assert_eq!(
        render_form(r#"{{ f | form(field="name") }}"#, form),
        "<CSRF>\n<NAME>",
        "csrf before the first"
    );
}

#[test]
fn an_empty_honeypot_adds_nothing() {
    let form = two_field_form("");
    assert_eq!(
        render_form(r#"{{ f | form(field="email") }}"#, form),
        "<EMAIL>"
    );
}

#[test]
fn the_whole_form_is_rebuilt_from_its_fields_with_the_trap() {
    let out = render_form("{{ f | form }}", two_field_form("<TRAP>"));
    assert!(
        out.contains("<NAME>") && out.contains("<EMAIL>") && out.ends_with("<TRAP>"),
        "{out}"
    );
}
