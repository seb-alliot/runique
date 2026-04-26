// Tests pour HiddenField (champs cachés et CSRF)

use runique::forms::base::FormField;
use runique::forms::fields::hidden::HiddenField;
use std::sync::Arc;
use tera::Tera;

fn minimal_tera() -> Arc<Tera> {
    let mut tera = Tera::default();
    tera.add_raw_template("base_hidden", "").unwrap();
    tera.add_raw_template("csrf", "").unwrap();
    Arc::new(tera)
}

// ── Constructeurs ──────────────────────────────────────────────────────────────

#[test]
fn test_hidden_field_new_csrf_structure() {
    let field = HiddenField::new_csrf();
    assert_eq!(field.base.name, "csrf_token");
    assert_eq!(field.base.type_field, "hidden");
    assert!(field.expected_value.is_none());
}

#[test]
fn test_hidden_field_new_structure() {
    let field = HiddenField::new("mon_champ");
    assert_eq!(field.base.name, "mon_champ");
    assert_eq!(field.base.type_field, "hidden");
    assert!(field.expected_value.is_none());
}

#[test]
fn test_hidden_field_label_builder() {
    let field = HiddenField::new("champ").label("Mon Label");
    assert_eq!(field.base.label, "Mon Label");
}

// ── set_expected_value ─────────────────────────────────────────────────────────

#[test]
fn test_set_expected_value() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("secret123");
    assert_eq!(field.expected_value, Some("secret123".to_string()));
}

// ── Validation : champ non-CSRF ────────────────────────────────────────────────

#[test]
fn test_non_csrf_field_toujours_valide() {
    let mut field = HiddenField::new("autre_champ");
    field.set_value("n-importe-quoi");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_non_csrf_field_vide_valide() {
    let mut field = HiddenField::new("autre_champ");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

// ── Validation : champ CSRF sans expected_value ────────────────────────────────

#[test]
fn test_csrf_sans_expected_value_passe() {
    let mut field = HiddenField::new_csrf();
    field.set_value("n-importe-quoi");
    // Sans expected_value défini, la validation passe toujours
    assert!(field.validate());
    assert!(field.error().is_none());
}

// ── Validation : champ CSRF avec expected_value ────────────────────────────────

#[test]
fn test_csrf_valeur_correcte_passe() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token_secret");
    field.set_value("token_secret");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_csrf_valeur_incorrecte_echoue() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token_secret");
    field.set_value("mauvais_token");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_csrf_valeur_vide_echoue() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token_secret");
    field.set_value("");
    assert!(!field.validate());
    let err = field.error().unwrap();
    assert!(err.to_lowercase().contains("csrf") || err.to_lowercase().contains("manquant"));
}

#[test]
fn test_csrf_message_erreur_valeur_invalide() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token_secret");
    field.set_value("mauvais");
    field.validate();
    let err = field.error().unwrap();
    assert!(err.to_lowercase().contains("invalid") || err.to_lowercase().contains("invalide"));
}

// ── Validation : effacement d'erreur ──────────────────────────────────────────

#[test]
fn test_csrf_clear_error_apres_correction() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token");
    field.set_value("mauvais");
    field.validate();
    assert!(field.error().is_some());

    // Correction de la valeur
    field.set_value("token");
    field.validate();
    assert!(field.error().is_none());
}

// ── Trait FormField : getters de base ─────────────────────────────────────────

#[test]
fn test_formfield_trait_name() {
    let field = HiddenField::new("my_field");
    assert_eq!(field.name(), "my_field");
}

#[test]
fn test_formfield_trait_field_type() {
    let field = HiddenField::new_csrf();
    assert_eq!(field.field_type(), "hidden");
}

#[test]
fn test_formfield_trait_set_value_get_value() {
    let mut field = HiddenField::new("champ");
    field.set_value("valeur_test");
    assert_eq!(field.value(), "valeur_test");
}

#[test]
fn test_formfield_trait_no_error_initial() {
    let field = HiddenField::new("champ");
    assert!(field.error().is_none());
}

// ── render() ──────────────────────────────────────────────────────────────────

#[test]
fn test_hidden_field_render_ok() {
    let tera = minimal_tera();
    let field = HiddenField::new("token");
    assert!(field.render(&tera).is_ok());
}

#[test]
fn test_csrf_field_render_ok() {
    let tera = minimal_tera();
    let field = HiddenField::new_csrf();
    assert!(field.render(&tera).is_ok());
}

#[test]
fn test_hidden_field_render_missing_template_err() {
    let tera = Arc::new(Tera::default());
    let field = HiddenField::new("token");
    assert!(field.render(&tera).is_err());
}
