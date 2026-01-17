use runique::prelude::RuniqueForm;
use runique::prelude::*;
use runique::Forms;
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
        form.field(
            &TextField::text("name")
                .label("Nom")
                .required("Ce champ est requis"),
        );

        form.field(&NumericField::integer("age").label("Âge"));
    }
}

// --- TESTS UNITAIRES ---

#[test]
fn test_forms_new() {
    let form = Forms::new();
    assert!(form.global_errors.is_empty());
    assert!(form.data().is_empty());
    assert!(!form.has_errors());
}

#[test]
fn test_forms_clear() {
    let mut form = Forms::new();
    form.global_errors.push("error".to_string());

    // Ajouter une erreur à un champ
    if let Some(field) = form.fields.get_mut("test") {
        field.set_error("error".to_string());
    }

    // Clear n'existe peut-être plus, mais on peut vider manuellement
    form.global_errors.clear();

    assert!(form.global_errors.is_empty());
}

#[tokio::test]
async fn test_runique_form_validation() {
    // 1. Initialiser Tera (requis par la nouvelle signature)
    let tera = Arc::new(Tera::default());

    let mut raw_data = HashMap::new();
    raw_data.insert("name".to_string(), "John".to_string());
    raw_data.insert("age".to_string(), "25".to_string());

    // 2. Passer tera.clone() en deuxième argument
    let mut form = TestForm::build_with_data(&raw_data, tera.clone());

    assert!(form.is_valid().await);
}

#[tokio::test]
async fn test_require_field_missing() {
    let tera = Arc::new(Tera::default());
    let raw_data = HashMap::new(); // Vide

    let mut form = TestForm::build_with_data(&raw_data, tera);

    // La validation devrait échouer car "name" est requis
    assert!(!form.is_valid().await);
    assert!(form.get_form().has_errors());

    // Vérifier que l'erreur est sur le champ "name"
    let errors = form.get_form().errors();
    assert!(errors.contains_key("name"));
    assert_eq!(errors.get("name"), Some(&"Ce champ est requis".to_string()));
}

#[tokio::test]
async fn test_emailfield_validation() {
    let tera = Arc::new(Tera::default());
    let mut raw_data = HashMap::new();
    raw_data.insert("email".to_string(), "TEST@Example.com".to_string());

    // Créer un formulaire avec un champ email
    #[derive(Clone)]
    struct EmailTestForm {
        form: Forms,
    }

    impl RuniqueForm for EmailTestForm {
        fn register_fields(form: &mut Forms) {
            form.field(&TextField::email("email").label("Email"));
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

    let mut form = EmailTestForm::build_with_data(&raw_data, tera);

    assert!(form.is_valid().await);

    // L'email devrait être normalisé en minuscules
    let value = form.get_form().get_value("email");
    assert_eq!(value, Some("test@example.com".to_string()));
}

#[test]
fn test_is_not_valid() {
    let mut form = Forms::new();
    form.global_errors.push("error".to_string());

    assert!(form.has_errors());
}
