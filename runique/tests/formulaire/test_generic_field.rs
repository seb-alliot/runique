// Tests pour GenericField — délégation via FieldKind

use runique::forms::base::FormField;
use runique::forms::fields::{HiddenField, TextField};
use runique::forms::generic::GenericField;

// ── From<TextField> → GenericField ──────────────────────────────

#[test]
fn test_generic_from_textfield_name() {
    let gf = GenericField::from(TextField::text("mon_champ"));
    assert_eq!(gf.name(), "mon_champ");
}

#[test]
fn test_generic_from_textfield_label() {
    let mut tf = TextField::text("champ");
    tf.set_label("Mon Label");
    let gf = GenericField::from(tf);
    assert_eq!(gf.label(), "Mon Label");
}

#[test]
fn test_generic_from_textfield_value() {
    let mut tf = TextField::text("champ");
    tf.set_value("bonjour");
    let gf = GenericField::from(tf);
    assert_eq!(gf.value(), "bonjour");
}

#[test]
fn test_generic_from_textfield_required() {
    let mut tf = TextField::text("champ");
    tf.set_required(true, None);
    let gf = GenericField::from(tf);
    assert!(gf.required());
}

#[test]
fn test_generic_from_textfield_error_none() {
    let gf = GenericField::from(TextField::text("champ"));
    assert!(gf.error().is_none());
}

#[test]
fn test_generic_from_textfield_set_error() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_error("erreur test".to_string());
    assert_eq!(gf.error().unwrap(), "erreur test");
}

#[test]
fn test_generic_from_textfield_field_type() {
    let gf = GenericField::from(TextField::text("champ"));
    assert_eq!(gf.field_type(), "text");
}

#[test]
fn test_generic_from_textfield_placeholder() {
    let mut tf = TextField::text("champ");
    tf.set_placeholder("ex: valeur");
    let gf = GenericField::from(tf);
    assert_eq!(gf.placeholder(), "ex: valeur");
}

#[test]
fn test_generic_set_name() {
    let mut gf = GenericField::from(TextField::text("ancien"));
    gf.set_name("nouveau");
    assert_eq!(gf.name(), "nouveau");
}

#[test]
fn test_generic_set_label() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_label("Nouveau Label");
    assert_eq!(gf.label(), "Nouveau Label");
}

#[test]
fn test_generic_set_value() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_value("nouvelle valeur");
    assert_eq!(gf.value(), "nouvelle valeur");
}

#[test]
fn test_generic_validate_required_empty_invalid() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_required(true, None);
    assert!(!gf.validate());
}

#[test]
fn test_generic_validate_required_filled_valid() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_required(true, None);
    gf.set_value("contenu");
    assert!(gf.validate());
}

// ── From<HiddenField> → GenericField ────────────────────────────

#[test]
fn test_generic_from_hidden_field_type() {
    let gf = GenericField::from(HiddenField::new("token"));
    assert_eq!(gf.field_type(), "hidden");
}

#[test]
fn test_generic_from_hidden_name() {
    let gf = GenericField::from(HiddenField::new("mon_token"));
    assert_eq!(gf.name(), "mon_token");
}

// ── to_json ──────────────────────────────────────────────────────

#[test]
fn test_generic_to_json_value_not_null() {
    let gf = GenericField::from(TextField::text("champ"));
    let json = gf.to_json_value();
    assert!(!json.is_null());
}

#[test]
fn test_generic_to_json_required() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_required(true, None);
    let json = gf.to_json_required();
    // L'objet contient "choice: true"
    assert_eq!(json["choice"], serde_json::json!(true));
}

#[test]
fn test_generic_to_json_meta_not_null() {
    let gf = GenericField::from(TextField::text("champ"));
    let meta = gf.to_json_meta();
    assert!(!meta.is_null());
}

// ── set_readonly / set_disabled ──────────────────────────────────

#[test]
fn test_generic_set_readonly() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_readonly(true, None);
    // readonly ne change pas la validation sur un champ vide non-requis
    assert!(gf.validate());
}

#[test]
fn test_generic_set_disabled() {
    let mut gf = GenericField::from(TextField::text("champ"));
    gf.set_disabled(true, None);
    assert!(gf.validate());
}
