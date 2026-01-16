use crate::formulaire::builder_form::base_struct::FieldConfig;
use crate::formulaire::builder_form::option_field::BoolChoice;
use crate::formulaire::builder_form::trait_form::FormField;

use std::sync::Arc;
use tera::Context;
use validator::ValidateEmail; // Excellent choix

#[derive(Clone)]
pub struct EmailField {
    pub base: FieldConfig,
}

impl EmailField {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                name: name.to_string(),
                label: label.to_string(),
                value: String::new(),
                placeholder: "exemple@domaine.com".to_string(),
                error: None,
                is_required: BoolChoice {
                    choice: false,
                    message: None,
                },
                readonly: None,
                disabled: None,
                type_field: "email".to_string(),
                html_attributes: std::collections::HashMap::new(),
                template_name: "base_string".to_string(),
                extra_context: std::collections::HashMap::new(),
            },
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.base.is_required = BoolChoice {
            choice: true,
            message: if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            },
        };
        self
    }
}

impl FormField for EmailField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn label(&self) -> &str {
        &self.base.label
    }
    fn value(&self) -> &str {
        &self.base.value
    }
    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }
    fn get_error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }
    fn set_error(&mut self, message: String) {
        self.base.error = Some(message);
    }
    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn validate(&mut self) -> bool {
        let trimmed_value = self.base.value.trim();

        // 1. Check Required
        if self.base.is_required.choice && trimmed_value.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "L'adresse email est obligatoire".to_string());
            self.set_error(msg);
            return false;
        }

        // 2. Check Format avec la crate 'validator'
        if !trimmed_value.is_empty() && !trimmed_value.validate_email() {
            self.set_error("Format d'adresse email invalide".to_string());
            return false;
        }

        self.base.error = None;
        true
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }
    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn render(&self, tera: &Arc<tera::Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("input_type", "email");

        tera.render(&self.base.template_name, &context)
            .map_err(|e| format!("Erreur de rendu Email: {}", e))
    }
}
