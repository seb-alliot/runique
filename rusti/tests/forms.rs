use rusti::formulaire::{
    formsrusti::{Forms, FormulaireTrait},
    field::{CharField, IntegerField, EmailField},
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct TestForm {
    form: Forms,
}

impl TestForm {
    fn new() -> Self {
        Self {
            form: Forms::new(),
        }
    }
}

impl FormulaireTrait for TestForm {
    fn new() -> Self {
        Self::new()
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.form.require("name", &CharField { allow_blank: false }, raw_data);
        self.form.optional("age", &IntegerField, raw_data);
        self.form.is_valid()
    }
}

#[test]
fn test_forms_new() {
    let form = Forms::new();
    assert!(form.errors.is_empty());
    assert!(form.cleaned_data.is_empty());
    assert!(form.is_valid());
}

#[test]
fn test_forms_clear() {
    let mut form = Forms::new();
    form.errors.insert("test".to_string(), "error".to_string());
    form.cleaned_data.insert("test".to_string(), serde_json::json!("value"));

    form.clear();

    assert!(form.errors.is_empty());
    assert!(form.cleaned_data.is_empty());
}

#[test]
fn test_charfield_validation() {
    let mut form = Forms::new();
    let mut raw_data = HashMap::new();
    raw_data.insert("name".to_string(), "John Doe".to_string());

    let field = CharField { allow_blank: false };
    form.field("name", &field, "John Doe");

    assert!(form.is_valid());
    let value: Option<String> = form.get_value("name");
    assert_eq!(value, Some("John Doe".to_string()));
}

// Note: CharField n'a pas de max_length dans l'implémentation actuelle
// Ce test est commenté car la fonctionnalité n'existe pas encore
// #[test]
// fn test_charfield_max_length() {
//     let mut form = Forms::new();
//     let long_string = "a".repeat(101);
//
//     let field = CharField { allow_blank: false };
//     form.field("name", &field, &long_string);
//
//     assert!(!form.is_valid());
//     assert!(form.errors.contains_key("name"));
// }

#[test]
fn test_charfield_blank_not_allowed() {
    let mut form = Forms::new();

    let field = CharField { allow_blank: false };
    form.field("name", &field, "");

    assert!(!form.is_valid());
    assert!(form.errors.contains_key("name"));
}

#[test]
fn test_charfield_blank_allowed() {
    let mut form = Forms::new();

    let field = CharField { allow_blank: true };
    form.field("name", &field, "");

    assert!(form.is_valid());
}

#[test]
fn test_integerfield_validation() {
    let mut form = Forms::new();

    let field = IntegerField;
    form.field("age", &field, "25");

    assert!(form.is_valid());
    let value: Option<i64> = form.get_value("age");
    assert_eq!(value, Some(25));
}

#[test]
fn test_integerfield_invalid_number() {
    let mut form = Forms::new();

    let field = IntegerField;
    form.field("age", &field, "not-a-number");

    assert!(!form.is_valid());
    assert!(form.errors.contains_key("age"));
}

// Note: IntegerField n'a pas de min_value/max_value dans l'implémentation actuelle
// Ces tests sont commentés car la fonctionnalité n'existe pas encore
// #[test]
// fn test_integerfield_min_value() { ... }
// #[test]
// fn test_integerfield_max_value() { ... }

#[test]
fn test_emailfield_validation() {
    let mut form = Forms::new();

    let field = EmailField;
    form.field("email", &field, "test@example.com");

    assert!(form.is_valid());
    let value: Option<String> = form.get_value("email");
    assert_eq!(value, Some("test@example.com".to_string()));
}

#[test]
fn test_emailfield_invalid() {
    let mut form = Forms::new();

    let field = EmailField;
    form.field("email", &field, "not-an-email");

    assert!(!form.is_valid());
    assert!(form.errors.contains_key("email"));
}

#[test]
fn test_require_field() {
    let mut form = Forms::new();
    let mut raw_data = HashMap::new();
    raw_data.insert("name".to_string(), "John".to_string());

    let field = CharField { allow_blank: false };
    form.require("name", &field, &raw_data);

    assert!(form.is_valid());
}

#[test]
fn test_require_field_missing() {
    let mut form = Forms::new();
    let raw_data = HashMap::new();

    let field = CharField { allow_blank: false };
    form.require("name", &field, &raw_data);

    assert!(!form.is_valid());
    assert!(form.errors.contains_key("name"));
    assert_eq!(form.errors.get("name"), Some(&"Requis".to_string()));
}

#[test]
fn test_optional_field() {
    let mut form = Forms::new();
    let mut raw_data = HashMap::new();
    raw_data.insert("phone".to_string(), "123456789".to_string());

    let field = CharField { allow_blank: true };
    form.optional("phone", &field, &raw_data);

    assert!(form.is_valid());
}

#[test]
fn test_optional_field_missing() {
    let mut form = Forms::new();
    let raw_data = HashMap::new();

    let field = CharField { allow_blank: true };
    form.optional("phone", &field, &raw_data);

    // Optional field missing should not cause an error
    assert!(form.is_valid());
}

#[test]
fn test_formulairetrait_validate() {
    let mut form = TestForm::new();
    let mut raw_data = HashMap::new();
    raw_data.insert("name".to_string(), "John".to_string());
    raw_data.insert("age".to_string(), "25".to_string());

    let is_valid = form.validate(&raw_data);
    assert!(is_valid);
    assert!(form.form.is_valid());
}

#[test]
fn test_get_value_typed() {
    let mut form = Forms::new();

    let field = IntegerField;
    form.field("age", &field, "30");

    let age: Option<i64> = form.get_value("age");
    assert_eq!(age, Some(30));

    let name_field = CharField { allow_blank: false };
    form.field("name", &name_field, "John");

    let name: Option<String> = form.get_value("name");
    assert_eq!(name, Some("John".to_string()));
}

#[test]
fn test_get_value_nonexistent() {
    let form = Forms::new();

    let value: Option<String> = form.get_value("nonexistent");
    assert_eq!(value, None);
}

#[test]
fn test_is_not_valid() {
    let mut form = Forms::new();
    form.errors.insert("test".to_string(), "error".to_string());

    assert!(form.is_not_valid());
    assert!(!form.is_valid());
}
