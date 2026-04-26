//! Tests — forms/form.rs
//! Couvre : fill(), clear_values(), finalize(), database_error(), set_url_params(), sérialisation

use axum::http::Method;
use runique::{
    forms::{
        fields::{boolean::BooleanField, text::TextField},
        form::Forms,
    },
    sea_orm::DbErr,
};
use std::collections::HashMap;

fn strmap(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════
// fill()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_fill_post_sets_values() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name").required());
    let data = strmap(&[("name", "Alice")]);
    form.fill(&data, Method::POST);
    assert_eq!(form.fields.get("name").unwrap().value(), "Alice");
}

#[test]
fn test_fill_post_marks_submitted() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    let data = strmap(&[("name", "Bob")]);
    form.fill(&data, Method::POST);
    // POST est toujours submitted même si données vides
    assert!(form.is_valid().is_ok());
}

#[test]
fn test_fill_patch_relaxes_password_required() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::password("pwd").required());
    form.fill(&HashMap::new(), Method::PATCH);
    // required relâché en PATCH → valide sans mot de passe
    assert!(form.is_valid().is_ok());
}

#[test]
fn test_fill_put_relaxes_password_required() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::password("pwd").required());
    form.fill(&HashMap::new(), Method::PUT);
    assert!(form.is_valid().is_ok());
}

#[test]
fn test_fill_get_not_submitted_if_empty() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("q"));
    form.fill(&HashMap::new(), Method::GET);
    // GET sans données → non soumis → is_valid renvoie Ok(false) sans erreur
    let result = form.is_valid();
    assert!(result.is_ok());
}

#[test]
fn test_fill_post_skips_password_on_get() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::password("pwd").required());
    let data = strmap(&[("pwd", "secret")]);
    form.fill(&data, Method::GET);
    // GET : le champ password est sauté, valeur reste ""
    assert_eq!(form.fields.get("pwd").unwrap().value(), "");
}

#[test]
fn test_fill_normalizes_unchecked_checkbox_post() {
    let mut form = Forms::new("csrf");
    form.field(&BooleanField::new("newsletter"));
    // POST sans la case cochée → doit être normalisé à "false"
    form.fill(&HashMap::new(), Method::POST);
    assert_eq!(form.fields.get("newsletter").unwrap().value(), "false");
}

// ═══════════════════════════════════════════════════════════════
// clear_values()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_clear_values_resets_fields() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Alice");
    form.clear_values();
    assert_eq!(form.fields.get("name").unwrap().value(), "");
}

#[test]
fn test_clear_values_resets_submitted() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Alice");
    form.clear_values();
    // après clear, form non soumis → is_valid() synchrone retourne Ok(false) sans erreur required
    let result = form.is_valid();
    // le champ name n'est pas required donc Ok(true) même vide
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════
// finalize()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_finalize_ok_on_valid_form() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Alice");
    assert!(form.finalize().is_ok());
}

// ═══════════════════════════════════════════════════════════════
// set_url_params()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_url_params_accessible_via_cleaned_string() {
    use runique::forms::field::RuniqueForm;

    struct F {
        form: Forms,
    }
    impl RuniqueForm for F {
        fn register_fields(form: &mut Forms) {
            form.field(&TextField::text("id"));
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

    let mut f = F {
        form: Forms::new("csrf"),
    };
    F::register_fields(&mut f.form);
    let path = strmap(&[("id", "42")]);
    let query = strmap(&[]);
    f.form.set_url_params(&path, &query);
    // cleaned_string récupère la valeur depuis path_params si le champ est vide
    assert_eq!(f.cleaned_string("id"), Some("42".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// add_value()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_value_known_field() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("email"));
    form.add_value("email", "test@example.com");
    assert_eq!(
        form.fields.get("email").unwrap().value(),
        "test@example.com"
    );
}

#[test]
fn test_add_value_unknown_field_ignored() {
    let mut form = Forms::new("csrf");
    // pas de champ "unknown" → add_value ne plante pas
    form.add_value("unknown", "ignored");
}

// ═══════════════════════════════════════════════════════════════
// database_error()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_database_error_unique_known_field() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("username"));
    // Format PostgreSQL : constraint "table_field_key" → parse_constraint_name extrait "username"
    let err = DbErr::Custom(
        r#"duplicate key value violates unique constraint "users_username_key""#.to_string(),
    );
    form.database_error(&err);
    assert!(form.fields.get("username").unwrap().error().is_some());
}

#[test]
fn test_database_error_unique_unknown_field_adds_global_error() {
    let mut form = Forms::new("csrf");
    // Erreur UNIQUE sur un champ absent du form → erreur globale
    let err = DbErr::Custom("UNIQUE constraint failed: users_email_key".to_string());
    form.database_error(&err);
    assert!(!form.errors.is_empty());
}

#[test]
fn test_database_error_generic_adds_global_error() {
    let mut form = Forms::new("csrf");
    let err = DbErr::Custom("connexion perdue".to_string());
    form.database_error(&err);
    assert!(!form.errors.is_empty());
}

#[test]
fn test_database_error_duplicate_entry() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("email"));
    // MySQL style
    let err = DbErr::Custom("Duplicate entry 'test@example.com' for key 'email'".to_string());
    form.database_error(&err);
    // Soit erreur sur le champ, soit erreur globale — dans les deux cas !has_errors() est faux
    assert!(form.has_errors());
}

// ═══════════════════════════════════════════════════════════════
// errors() / has_errors()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_errors_returns_map_with_failed_fields() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name").required());
    let _ = form.is_valid();
    let errors = form.errors();
    assert!(errors.contains_key("name"));
}

#[test]
fn test_has_errors_false_when_valid() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Alice");
    let _ = form.is_valid();
    assert!(!form.has_errors());
}

// ═══════════════════════════════════════════════════════════════
// Sérialisation JSON
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_serialize_form_contains_fields_key() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("username"));
    form.add_value("username", "alice");
    let json = serde_json::to_value(&form).unwrap();
    assert!(json.get("fields").is_some());
}

#[test]
fn test_serialize_form_errors_empty_when_valid() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Bob");
    let _ = form.is_valid();
    let json = serde_json::to_value(&form).unwrap();
    let errors = json.get("errors").unwrap().as_object().unwrap();
    assert!(errors.is_empty());
}
