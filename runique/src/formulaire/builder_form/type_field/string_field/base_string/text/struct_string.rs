use crate::formulaire::validation_form::builder_form::trait_form::FormField;
use crate::formulaire::validation_form::builder_form::base_struct::{TextConfig, FieldConfig};
use crate::formulaire::validation_form::builder_form::option_struct::IsRequired;
use serde_json::{json, Value};

pub struct TextField {
    pub config: TextConfig,
}

impl TextField {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            config: TextConfig {
                base: FieldConfig {
                    id: format!("id_{}", name),
                    name: name.to_string(),
                    label: label.to_string(),
                    value: String::new(),
                    placeholder: String::new(),
                    is_required: IsRequired { choice: false, message: None },
                    error: None,
                    readonly: None,
                    disabled: None,
                    type_field: "text".to_string(),
                    html_attributes: std::collections::HashMap::new(),
                    template_name: "forms/text.html".to_string(),
                    extra_context: std::collections::HashMap::new(),
                },
                max_length: crate::formulaire::validation_form::builder_form::option_struct::MaxlenghtConfig {
                    max_length: 255,
                },
            },
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.config.base.placeholder = p.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.config.base.is_required = IsRequired {
            choice: true,
            message: Some(msg.to_string()),
        };
        self
    }
}

impl FormField for TextField {
    fn id(&self) -> &str { &self.config.base.id }
    fn name(&self) -> &str { &self.config.base.name }
    fn label(&self) -> &str { &self.config.base.label }
    fn value(&self) -> &str { &self.config.base.value }

    fn get_json_value(&self) -> Value {
        json!(self.config.base.value)
    }

    fn get_error(&self) -> Option<&String> {
        self.config.base.error.as_ref()
    }

    fn set_error(&mut self, message: String) {
        self.config.base.error = Some(message);
    }

    fn set_value(&mut self, value: &str) {
        self.config.base.value = value.to_string();
    }

    fn validate(&mut self) -> bool {
        // Logique de validation simple
        if self.config.base.is_required.choice && self.config.base.value.is_empty() {
            let msg = self.config.base.is_required.message.clone()
                .unwrap_or_else(|| "Ce champ est requis".to_string());
            self.set_error(msg);
            return false;
        }

        if self.config.base.value.len() > self.config.max_length.max_length {
            self.set_error(format!("Trop long (max {} caractères)", self.config.max_length.max_length));
            return false;
        }

        self.config.base.error = None;
        true
    }

    fn render(&self) -> Result<String, String> {
        // Ici, on utilisera plus tard Tera pour rendre le template
        // Pour l'instant, on simule le HTML
        Ok(format!(
            "<label for='{}'>{}</label><input type='text' name='{}' value='{}'>",
            self.id(), self.label(), self.name(), self.value()
        ))
    }
}