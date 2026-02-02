use crate::forms::base::*;
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone, Serialize, Debug)]
pub struct BooleanField {
    pub base: FieldConfig,
}

impl CommonFieldConfig for BooleanField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl BooleanField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "checkbox", "base_boolean"),
        }
    }

    pub fn radio(name: &str) -> Self {
        let mut field = Self::new(name);
        field.base.type_field = "radio".to_string();
        field
    }

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
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());

        // Ajouter l'état "checked"
        let is_checked = self.base.value == "true";
        context.insert("checked", &is_checked);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}
