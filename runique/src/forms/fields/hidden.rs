use crate::forms::field::FormField;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone, Serialize, Debug)]
pub struct HiddenField {
    pub name: String,
    pub value: String,
    pub input_type: String,
    pub template_name: String,
    /// Token de session attendu (pour validation CSRF)
    pub expected_value: Option<String>,
    /// Message d'erreur
    pub error_message: Option<String>,
}

impl HiddenField {
    /// Constructeur spécifique pour un champ caché CSRF
    pub fn new_csrf() -> Self {
        Self {
            name: "csrf_token".to_string(),
            value: String::new(),
            input_type: "hidden".to_string(),
            template_name: "csrf".to_string(),
            expected_value: None,
            error_message: None,
        }
    }

    /// Définit la valeur attendue pour la validation (token de session)
    pub fn set_expected_value(&mut self, expected: &str) {
        self.expected_value = Some(expected.to_string());
    }

    pub fn set_value(&mut self, token: &str) {
        self.value = token.to_string();
    }
}

impl FormField for HiddenField {
    fn name(&self) -> &str {
        &self.name
    }
    fn template_name(&self) -> &str {
        &self.template_name
    }

    fn label(&self) -> &str {
        "" // champ caché, pas de label
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn placeholder(&self) -> &str {
        ""
    }

    fn field_type(&self) -> &str {
        &self.input_type
    }

    fn error(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn set_label(&mut self, _label: &str) {}
    fn set_placeholder(&mut self, _placeholder: &str) {}
    fn set_value(&mut self, value: &str) {
        self.value = value.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.error_message = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_readonly(&mut self, _readonly: bool, _msg: Option<&str>) {}
    fn set_disabled(&mut self, _disabled: bool, _msg: Option<&str>) {}
    fn set_required(&mut self, _required: bool, _msg: Option<&str>) {}
    fn set_html_attribute(&mut self, _key: &str, _value: &str) {}

    fn validate(&mut self) -> bool {
        // Pour un champ CSRF, vérifier que la valeur correspond à celle attendue
        if self.name == "csrf_token" {
            if let Some(expected) = &self.expected_value {
                if self.value.trim().is_empty() {
                    self.set_error("Token CSRF manquant".to_string());
                    return false;
                }

                if self.value != *expected {
                    self.set_error("Token CSRF invalide".to_string());
                    return false;
                }
            }
        }

        self.set_error(String::new());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert(
            "field",
            &json!({
                "name": self.name,
                "value": self.value,
                "input_type": self.input_type
            }),
        );
        println!("Rendering HiddenField with context: {:?}", context);

        tera.render(&self.template_name, &context)
            .map_err(|e| e.to_string())
    }
}
