use crate::formulaire::validation_form::builder_form::trait_form::FormField;
use crate::formulaire::validation_form::builder_form::base_struct::FieldConfig;

use crate::app::TERA;
use tera::Context;
use validator::ValidateEmail;

pub struct EmailField {
    pub base: FieldConfig,
}

impl EmailField {
    pub fn new(id: &str, name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                id: id.to_string(),
                name: name.to_string(),
                label: label.to_string(),
                value: String::new(),
                placeholder: "exemple@domaine.com".to_string(),
                error: None,
                is_required: false,
            },
        }
    }

    pub fn required(mut self) -> Self {
        self.base.is_required = true;
        self
    }
}

impl FormField for EmailField {
    fn id(&self) -> &str { &self.base.id }
    fn name(&self) -> &str { &self.base.name }
    fn label(&self) -> &str { &self.base.label }
    fn value(&self) -> &str { &self.base.value }
    fn placeholder(&self) -> &str { &self.base.placeholder }
    fn get_error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn validate(&mut self) -> bool {
        let trimmed_value = self.base.value.trim();
        if self.base.is_required && trimmed_value.is_empty() {
            self.base.error = Some("L'adresse email est obligatoire".to_string());
            return false;
        }

        if !trimmed_value.is_empty() {
            if !trimmed_value.validate_email() {
                self.base.error = Some("Format d'adresse email invalide".to_string());
                return false;
            }
        }

        self.base.error = None;
        true
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn render(&self) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("id", &self.base.id);
        context.insert("name", &self.base.name);
        context.insert("label", &self.base.label);
        context.insert("value", &self.base.value);
        context.insert("placeholder", &self.base.placeholder);
        context.insert("error", &self.base.error);
        context.insert("required", &self.base.is_required);

        context.insert("input_type", "email");

        TERA.render("base_string", &context)
            .map_err(|e| format!("Erreur de rendu Email (id: {}): {}", self.base.id, e))
    }
}