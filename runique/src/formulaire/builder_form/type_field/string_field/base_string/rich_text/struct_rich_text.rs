use crate::formulaire::validation_form::builder_form::trait_form::FormField;
use crate::formulaire::validation_form::builder_form::base_struct::FieldConfig;

use crate::app::TERA;
use tera::Context;

pub struct RichTextField {
    pub base : FieldConfig,
    pub editor_config: String,
}

impl RichTextField {
    pub fn new(id: &str, name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                id: id.to_string(),
                name: name.to_string(),
                label: label.to_string(),
                value: String::new(),
                placeholder: String::new(),
                is_required: false,
                error: None,
            },
            editor_config: "default".to_string(),
        }
    }

    pub fn config(mut self, config: &str) -> Self {
        self.editor_config = config.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.base.is_required = true;
        self
    }
}

impl FormField for RichTextField {
    fn id(&self) -> &str { &self.base.id }
    fn name(&self) -> &str { &self.base.name }
    fn label(&self) -> &str { &self.base.label }
    fn value(&self) -> &str { &self.base.value }
    fn placeholder(&self) -> &str { &self.base.placeholder }
    fn get_error(&self) -> Option<&String> { self.base.error.as_ref() }

    fn validate(&mut self) -> bool {
        if self.base.is_required && self.base.value.trim().is_empty() {
            self.base.error = Some("Le contenu riche est obligatoire".to_string());
            return false;
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
        context.insert("editor_config", &self.editor_config);

        TERA.render("rich_text", &context)
            .map_err(|e| format!("Erreur rendu RichText ({}): {}", self.base.id, e))
    }
}