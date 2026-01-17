use crate::formulaire::builder_form::base_struct::*;
use crate::formulaire::builder_form::option_field::BoolChoice;
use crate::formulaire::builder_form::trait_form::FormField;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone, Serialize)]
pub struct BooleanField {
    pub base: FieldConfig,
}

impl BooleanField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig {
                name: name.to_string(),
                label: String::new(),
                value: "false".to_string(),
                placeholder: String::new(),
                is_required: BoolChoice::default(),
                error: None,
                type_field: "checkbox".to_string(),
                html_attributes: HashMap::new(),
                template_name: "base_boolean".to_string(),
                extra_context: HashMap::new(),
            },
        }
    }

    // Constructeur pour radio buttons
    pub fn radio(name: &str) -> Self {
        let mut field = Self::new(name);
        field.base.type_field = "radio".to_string();
        field
    }

    // Builder methods
    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn checked(mut self) -> Self {
        self.base.value = "true".to_string();
        self
    }

    pub fn unchecked(mut self) -> Self {
        self.base.value = "false".to_string();
        self
    }
}

impl FormField for BooleanField {
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

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn is_required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = match value.to_lowercase().as_str() {
            "true" | "1" | "on" | "yes" => "true",
            _ => "false",
        }
        .to_string();
    }

    fn set_placeholder(&mut self, _p: &str) {}

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn validate(&mut self) -> bool {
        // Pour un champ booléen "requis", on vérifie qu'il est coché (true)
        if self.base.is_required.choice && self.base.value != "true" {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Vous devez accepter ce champ".into());
            self.set_error(msg);
            return false;
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("input_type", &self.base.type_field);

        // Ajouter l'état "checked"
        let is_checked = self.base.value == "true";
        context.insert("checked", &is_checked);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value == "true" || self.base.value == "1")
    }
}
