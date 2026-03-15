// Tests pour ModelForm — valeurs par défaut des méthodes `fields()` et `exclude()`

use runique::forms::model_form::ModelForm;
use runique::migration::schema::ModelSchema;

// Implémentation minimale pour tester les valeurs par défaut du trait
struct SimpleForm;

impl ModelForm for SimpleForm {
    fn schema() -> ModelSchema {
        ModelSchema::new("SimpleForm")
    }
}

// Implémentation avec surcharge de fields() et exclude()
struct FilteredForm;

impl ModelForm for FilteredForm {
    fn schema() -> ModelSchema {
        ModelSchema::new("FilteredForm")
    }

    fn fields() -> Option<&'static [&'static str]> {
        Some(&["name", "email"])
    }

    fn exclude() -> Option<&'static [&'static str]> {
        Some(&["password"])
    }
}

// ── Valeurs par défaut ────────────────────────────────────────────

#[test]
fn test_model_form_fields_default_none() {
    assert!(SimpleForm::fields().is_none());
}

#[test]
fn test_model_form_exclude_default_none() {
    assert!(SimpleForm::exclude().is_none());
}

// ── Surcharge ────────────────────────────────────────────────────

#[test]
fn test_model_form_fields_override() {
    let fields = FilteredForm::fields().unwrap();
    assert_eq!(fields, &["name", "email"]);
}

#[test]
fn test_model_form_exclude_override() {
    let excluded = FilteredForm::exclude().unwrap();
    assert!(excluded.contains(&"password"));
}

// ── schema() ─────────────────────────────────────────────────────

#[test]
fn test_model_form_schema_model_name() {
    let schema = SimpleForm::schema();
    assert_eq!(schema.model_name, "SimpleForm");
}

#[test]
fn test_model_form_schema_table_name_snake_case() {
    struct UserProfile;
    impl ModelForm for UserProfile {
        fn schema() -> ModelSchema {
            ModelSchema::new("UserProfile")
        }
    }
    let schema = UserProfile::schema();
    assert_eq!(schema.table_name, "user_profile");
}
