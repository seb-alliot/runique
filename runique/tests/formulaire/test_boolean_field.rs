//! Tests — forms/fields/boolean.rs : BooleanField builders et validate()

use runique::forms::base::FormField;
use runique::forms::fields::boolean::BooleanField;
use std::sync::Arc;
use tera::Tera;

fn minimal_tera() -> Arc<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_template("base_boolean", "").unwrap();
    Arc::new(tera)
}

#[test]
fn test_boolean_field_new() {
    let f = BooleanField::new("accept");
    assert_eq!(f.base.name, "accept");
    assert_eq!(f.base.type_field, "checkbox");
}

#[test]
fn test_boolean_field_radio() {
    let f = BooleanField::radio("gender");
    assert_eq!(f.base.type_field, "radio");
    assert_eq!(f.base.name, "gender");
}

#[test]
fn test_boolean_field_label() {
    let f = BooleanField::new("accept").label("J'accepte");
    assert_eq!(f.base.label, "J'accepte");
}

#[test]
fn test_boolean_field_checked() {
    let f = BooleanField::new("agree").checked();
    assert_eq!(f.base.value, "true");
}

#[test]
fn test_boolean_field_unchecked() {
    let f = BooleanField::new("agree").unchecked();
    assert_eq!(f.base.value, "false");
}

#[test]
fn test_boolean_field_required() {
    let f = BooleanField::new("tos").required();
    assert!(f.base.is_required.choice);
}

#[test]
fn test_boolean_field_validate_always_true() {
    let mut f = BooleanField::new("accept");
    f.set_value("false");
    assert!(f.validate());
    assert!(f.error().is_none());
}

#[test]
fn test_boolean_field_validate_clears_error() {
    let mut f = BooleanField::new("accept");
    // Même avec une valeur vide, validate() retourne true
    f.set_value("");
    assert!(f.validate());
}

#[test]
fn test_boolean_field_value_after_set() {
    let mut f = BooleanField::new("toggle");
    f.set_value("true");
    assert_eq!(f.value(), "true");
}

// ═══════════════════════════════════════════════════════════════
// render()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_boolean_field_render_ok() {
    let tera = minimal_tera();
    let f = BooleanField::new("accept").checked();
    assert!(f.render(&tera).is_ok());
}

#[test]
fn test_boolean_field_render_unchecked_ok() {
    let tera = minimal_tera();
    let f = BooleanField::new("accept").unchecked();
    assert!(f.render(&tera).is_ok());
}

#[test]
fn test_boolean_field_render_missing_template_err() {
    let tera = Arc::new(Tera::default());
    let f = BooleanField::new("accept");
    assert!(f.render(&tera).is_err());
}
