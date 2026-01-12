use runique::formulaire::{
    field_folder::fieldhtml::{CharField, EmailField, IntegerField},
    forms::{Forms, RuniqueForm},
};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

#[derive(Clone)]
struct TestForm {
    form: Forms,
}

// Implémentation du nouveau trait RuniqueForm
impl RuniqueForm for TestForm {
    fn from_form(form: Forms) -> Self {
        Self { form }
    }

    fn get_form(&self) -> &Forms {
        &self.form
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }

    // Cette méthode définit la structure du formulaire (Rendu HTML)
    fn register_fields(form: &mut Forms) {
        // Note: register_field nécessite maintenant Tera pour générer du HTML.
        // Dans les tests unitaires sans Tera, cela générera un message d'erreur dans fields_html
        // mais n'empêchera pas la validation de fonctionner.
        form.register_field("name", "Nom", &CharField { allow_blank: false });
        form.register_field("age", "Âge", &IntegerField);
    }

    // Cette méthode définit la logique de validation (Traitement des données)
    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
        form.require("name", &CharField { allow_blank: false }, raw_data);
        form.optional("age", &IntegerField, raw_data);
    }
}

// --- TESTS UNITAIRES ---

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
    form.cleaned_data
        .insert("test".to_string(), serde_json::json!("value"));

    form.clear();

    assert!(form.errors.is_empty());
    assert!(form.cleaned_data.is_empty());
}

#[test]
fn test_runique_form_validation() {
    // 1. Initialiser Tera (requis par la nouvelle signature)
    let tera = Arc::new(Tera::default());

    let mut raw_data = HashMap::new();
    raw_data.insert("name".to_string(), "John".to_string());
    raw_data.insert("age".to_string(), "25".to_string());

    // 2. Passer tera.clone() en deuxième argument
    let form = TestForm::build_with_current_data(&raw_data, tera.clone());

    assert!(form.is_valid());
}

#[test]
fn test_require_field_missing() {
    let mut form = Forms::new();
    let raw_data = HashMap::new(); // Vide

    let field = CharField { allow_blank: false };
    form.require("name", &field, &raw_data);

    assert!(!form.is_valid());
    assert!(form.errors.contains_key("name"));
    // On vérifie le message d'erreur défini dans forms.rs
    assert_eq!(
        form.errors.get("name"),
        Some(&"Ce champ est requis".to_string())
    );
}

#[test]
fn test_emailfield_validation() {
    let mut form = Forms::new();
    let field = EmailField;

    form.field("email", &field, "TEST@Example.com");

    assert!(form.is_valid());
    let value: Option<String> = form.get_value("email");
    assert_eq!(value, Some("test@example.com".to_string()));
}

#[test]
fn test_is_not_valid() {
    let mut form = Forms::new();
    form.errors.insert("test".to_string(), "error".to_string());

    assert!(form.is_not_valid());
    assert!(!form.is_valid());
}
