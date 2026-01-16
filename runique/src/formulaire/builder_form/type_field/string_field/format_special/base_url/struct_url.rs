use crate::formulaire::builder_form::base_struct::{FieldConfig, TextConfig};
use crate::formulaire::builder_form::option_field::{BoolChoice, LengthConstraint};
use crate::formulaire::builder_form::trait_form::FormField;

use std::collections::HashMap;
use std::sync::Arc;
use tera::Context;
use validator::ValidateUrl;

#[derive(Clone)]
pub struct URLField {
    pub config: TextConfig,
}

impl URLField {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            config: TextConfig {
                base: FieldConfig {
                    name: name.to_string(),
                    label: label.to_string(),
                    value: String::new(),
                    placeholder: "https://...".to_string(),
                    is_required: BoolChoice {
                        choice: false,
                        message: None,
                    },
                    error: None,
                    readonly: None,
                    disabled: None,
                    type_field: "url".to_string(),
                    html_attributes: HashMap::new(),
                    template_name: "base_string".to_string(),
                    extra_context: HashMap::new(),
                },
                // Limite standard pour les URL (2048 caractères)
                max_length: Some(LengthConstraint {
                    limit: 2048,
                    message: None,
                }),
            },
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.config.base.placeholder = p.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.config.base.is_required = BoolChoice {
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

impl FormField for URLField {
    fn name(&self) -> &str {
        &self.config.base.name
    }
    fn field_type(&self) -> &str {
        &self.config.base.type_field
    }
    fn label(&self) -> &str {
        &self.config.base.label
    }
    fn value(&self) -> &str {
        &self.config.base.value
    }
    fn placeholder(&self) -> &str {
        &self.config.base.placeholder
    }
    fn is_required(&self) -> bool {
        self.config.base.is_required.choice
    }
    fn get_error(&self) -> Option<&String> {
        self.config.base.error.as_ref()
    }

    fn set_error(&mut self, message: String) {
        self.config.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_value(&mut self, value: &str) {
        self.config.base.value = value.to_string();
    }
    fn set_placeholder(&mut self, placeholder: &str) {
        self.config.base.placeholder = placeholder.to_string();
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        self.config.base.readonly = Some(BoolChoice {
            choice: readonly,
            message: msg.map(|s| s.to_string()),
        });
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        self.config.base.disabled = Some(BoolChoice {
            choice: disabled,
            message: msg.map(|s| s.to_string()),
        });
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.config.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.config
            .base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn get_is_required_config(&self) -> serde_json::Value {
        serde_json::to_value(&self.config.base.is_required)
            .unwrap_or(serde_json::json!({"choice": false, "message": null}))
    }

    fn get_readonly_config(&self) -> serde_json::Value {
        if let Some(ref readonly) = self.config.base.readonly {
            serde_json::to_value(readonly)
                .unwrap_or(serde_json::json!({"choice": false, "message": null}))
        } else {
            serde_json::json!({"choice": false, "message": null})
        }
    }

    fn get_disabled_config(&self) -> serde_json::Value {
        if let Some(ref disabled) = self.config.base.disabled {
            serde_json::to_value(disabled)
                .unwrap_or(serde_json::json!({"choice": false, "message": null}))
        } else {
            serde_json::json!({"choice": false, "message": null})
        }
    }

    fn get_html_attributes(&self) -> serde_json::Value {
        serde_json::to_value(&self.config.base.html_attributes).unwrap_or(serde_json::json!({}))
    }

    fn validate(&mut self) -> bool {
        let val = self.config.base.value.trim();

        // 1. Validation de l'obligation
        if self.config.base.is_required.choice && val.is_empty() {
            let msg = self
                .config
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "L'URL est obligatoire".to_string());
            self.set_error(msg);
            return false;
        }

        // 2. Validation du format URL via la crate validator
        if !val.is_empty() && !val.validate_url() {
            self.set_error("Veuillez entrer une URL valide (ex: https://google.com)".to_string());
            return false;
        }

        self.set_error("".to_string());
        true
    }

    fn set_name(&mut self, name: &str) {
        self.config.base.name = name.to_string();
    }
    fn set_label(&mut self, label: &str) {
        self.config.base.label = label.to_string();
    }

    fn render(&self, tera: &Arc<tera::Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.config.base);
        context.insert("input_type", "url");

        tera.render(&self.config.base.template_name, &context)
            .map_err(|e| format!("Erreur rendu URL: {}", e))
    }
}
