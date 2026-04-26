// Tests pour FormValidator et ValidationError

use indexmap::IndexMap;
use runique::forms::base::FormField;
use runique::forms::fields::hidden::HiddenField;
use runique::forms::validator::{FormValidator, ValidationError};

// ── ValidationError Display ────────────────────────────────────────────────────

#[test]
fn test_validation_error_stackoverflow_display() {
    let err = ValidationError::StackOverflow;
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("stack") || msg.to_lowercase().contains("overflow"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_validation_error_field_validation_display() {
    let mut errors = std::collections::HashMap::new();
    errors.insert("username".to_string(), "Ce champ est requis".to_string());
    let err = ValidationError::FieldValidation(errors);
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("validation"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_validation_error_global_errors_display() {
    let err = ValidationError::GlobalErrors(vec!["Erreur 1".to_string(), "Erreur 2".to_string()]);
    let msg = err.to_string();
    assert!(msg.contains("Erreur 1"), "Message inattendu: {}", msg);
    assert!(msg.contains("Erreur 2"), "Message inattendu: {}", msg);
}

#[test]
fn test_validation_error_global_vide() {
    let err = ValidationError::GlobalErrors(vec![]);
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

// ── ValidationError implements std::error::Error ──────────────────────────────

#[test]
fn test_validation_error_is_std_error() {
    let err: &dyn std::error::Error = &ValidationError::StackOverflow;
    assert!(err.source().is_none());
}

#[test]
fn test_validation_error_clone() {
    let original = ValidationError::GlobalErrors(vec!["msg".to_string()]);
    let cloned = original.clone();
    assert_eq!(cloned.to_string(), original.to_string());
}

// ── FormValidator::has_errors ──────────────────────────────────────────────────

fn make_empty_fields() -> IndexMap<String, Box<dyn FormField>> {
    IndexMap::new()
}

fn make_fields_with_error() -> IndexMap<String, Box<dyn FormField>> {
    let mut map: IndexMap<String, Box<dyn FormField>> = IndexMap::new();
    let mut field = HiddenField::new("champ_test");
    field.set_error("Champ invalide".to_string());
    map.insert("champ_test".to_string(), Box::new(field));
    map
}

fn make_fields_without_error() -> IndexMap<String, Box<dyn FormField>> {
    let mut map: IndexMap<String, Box<dyn FormField>> = IndexMap::new();
    let field = HiddenField::new("champ_ok");
    map.insert("champ_ok".to_string(), Box::new(field));
    map
}

#[test]
fn test_has_errors_vide_sans_global() {
    let fields = make_empty_fields();
    assert!(!FormValidator::has_errors(&fields, &[]));
}

#[test]
fn test_has_errors_avec_erreur_globale() {
    let fields = make_empty_fields();
    let global = vec!["Erreur globale".to_string()];
    assert!(FormValidator::has_errors(&fields, &global));
}

#[test]
fn test_has_errors_avec_champ_en_erreur() {
    let fields = make_fields_with_error();
    assert!(FormValidator::has_errors(&fields, &[]));
}

#[test]
fn test_has_errors_champ_sans_erreur() {
    let fields = make_fields_without_error();
    assert!(!FormValidator::has_errors(&fields, &[]));
}

#[test]
fn test_has_errors_combine_champ_et_global() {
    let fields = make_fields_without_error();
    let global = vec!["Problème global".to_string()];
    assert!(FormValidator::has_errors(&fields, &global));
}

// ── FormValidator::collect_errors ─────────────────────────────────────────────

#[test]
fn test_collect_errors_aucune_erreur() {
    let fields = make_fields_without_error();
    let result = FormValidator::collect_errors(&fields, &[]);
    assert!(result.is_empty());
}

#[test]
fn test_collect_errors_erreur_de_champ() {
    let fields = make_fields_with_error();
    let result = FormValidator::collect_errors(&fields, &[]);
    assert!(result.contains_key("champ_test"));
    assert_eq!(result["champ_test"], "Champ invalide");
}

#[test]
fn test_collect_errors_erreur_globale() {
    let fields = make_empty_fields();
    let global = vec!["Erreur A".to_string(), "Erreur B".to_string()];
    let result = FormValidator::collect_errors(&fields, &global);
    assert!(result.contains_key("global"));
    let global_msg = &result["global"];
    assert!(global_msg.contains("Erreur A"));
    assert!(global_msg.contains("Erreur B"));
}

#[test]
fn test_collect_errors_combine() {
    let fields = make_fields_with_error();
    let global = vec!["Problème général".to_string()];
    let result = FormValidator::collect_errors(&fields, &global);
    assert!(result.contains_key("champ_test"));
    assert!(result.contains_key("global"));
}
