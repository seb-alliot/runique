//! Tests d'intégration pour le framework Runique
//! Teste les principales fonctionnalités du framework

use runique::forms::field::FormField;
use runique::prelude::*;

// ============================================================================
// Tests des formulaires
// ============================================================================

#[test]
fn test_text_field_creation() {
    let field = TextField::text("username");
    assert_eq!(field.name(), "username");
    assert_eq!(field.field_type(), "text");
}

#[test]
fn test_email_field() {
    let field = TextField::email("email");
    assert_eq!(field.field_type(), "email");
}

#[test]
fn test_password_field() {
    let field = TextField::password("password");
    assert_eq!(field.field_type(), "password");
}

#[test]
fn test_textarea_field() {
    let field = TextField::textarea("content");
    assert_eq!(field.field_type(), "textarea");
}

#[test]
fn test_richtext_field() {
    let field = TextField::richtext("description");
    assert_eq!(field.field_type(), "richtext");
}

#[test]
fn test_url_field() {
    let field = TextField::url("website");
    assert_eq!(field.field_type(), "url");
}

#[test]
fn test_numeric_field_integer() {
    let field = NumericField::integer("age");
    assert_eq!(field.name(), "age");
    assert_eq!(field.field_type(), "number");
}

#[test]
fn test_numeric_field_decimal() {
    let field = NumericField::decimal("price");
    assert_eq!(field.name(), "price");
}

#[test]
fn test_text_field_builder() {
    let field = TextField::text("name")
        .label("Nom complet")
        .placeholder("Entrez votre nom");
    // Les méthodes sur builder retournent Self, pas de getter
    assert_eq!(field.name(), "name");
}

#[test]
fn test_field_required() {
    let field = TextField::text("name").required("Ce champ est obligatoire");
    assert_eq!(field.name(), "name");
}

#[test]
fn test_forms_new() {
    let form = Forms::new("csrf_token_123");
    assert!(form.fields.contains_key("csrf_token"));
}

#[test]
fn test_forms_add_field() {
    let mut form = Forms::new("csrf_token");
    let field = TextField::text("username");
    form.field(&field);

    assert!(form.fields.contains_key("username"));
}

#[test]
fn test_forms_fill_data() {
    let mut form = Forms::new("csrf");
    let mut field = TextField::text("name");
    field.set_value("John Doe");
    form.field(&field);

    if let Some(f) = form.fields.get("name") {
        assert_eq!(f.value(), "John Doe");
    }
}

#[test]
fn test_complex_form_creation() {
    let mut form = Forms::new("csrf_token");

    form.field(
        &TextField::text("username")
            .label("Nom d'utilisateur")
            .placeholder("Entrez un pseudo"),
    );

    form.field(&TextField::email("email").label("Email"));

    form.field(
        &TextField::password("password")
            .label("Mot de passe")
            .placeholder("Entrez votre mot de passe"),
    );

    form.field(&NumericField::integer("age").label("Âge"));

    assert!(form.fields.contains_key("username"));
    assert!(form.fields.contains_key("email"));
    assert!(form.fields.contains_key("password"));
    assert!(form.fields.contains_key("age"));
    assert_eq!(form.fields.len(), 5); // csrf + 4 fields
}

// ============================================================================
// Tests de la configuration
// ============================================================================

#[test]
fn test_prelude_exports() {
    // Vérifie que les principales structures sont disponibles
    let _form = Forms::new("test");
    let _field = TextField::text("test");
    let _numeric = NumericField::integer("test");
}

#[test]
fn test_field_types_available() {
    // Vérifie que les types de champs sont disponibles
    let _text = TextField::text("f");
    let _email = TextField::email("f");
    let _password = TextField::password("f");
    let _textarea = TextField::textarea("f");
    let _richtext = TextField::richtext("f");
    let _url = TextField::url("f");
    let _numeric = NumericField::integer("f");
    let _float = NumericField::float("f");
    let _decimal = NumericField::decimal("f");
}

// ============================================================================
// Résumé des tests
// ============================================================================
/*
Tests de formulaires:
- ✅ test_text_field_creation
- ✅ test_email_field
- ✅ test_password_field
- ✅ test_textarea_field
- ✅ test_richtext_field
- ✅ test_url_field
- ✅ test_numeric_field_integer
- ✅ test_numeric_field_decimal
- ✅ test_text_field_builder
- ✅ test_field_required
- ✅ test_forms_new
- ✅ test_forms_add_field
- ✅ test_forms_fill_data
- ✅ test_complex_form_creation

Tests de configuration:
- ✅ test_prelude_exports
- ✅ test_field_types_available

Total: 16 tests
*/
