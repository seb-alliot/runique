use crate::formulaire::builder_form::base_struct::{FieldConfig, TextConfig};
use crate::formulaire::builder_form::option_field::{BoolChoice, LengthConstraint};
use crate::formulaire::builder_form::trait_form::FormField;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone)]
pub struct TextAreaField {
    pub config: TextConfig,
}

impl TextAreaField {
    pub fn new(name: &str, label: &str, placeholder: &str) -> Self {
        Self {
            config: TextConfig {
                base: FieldConfig {
                    name: name.to_string(),
                    label: label.to_string(),
                    value: String::new(),
                    placeholder: placeholder.to_string(),
                    is_required: BoolChoice {
                        choice: false,
                        message: None,
                    },
                    error: None,
                    readonly: None,
                    disabled: None,
                    type_field: "textarea".to_string(),
                    html_attributes: HashMap::new(),
                    template_name: "base_string".to_string(),
                    extra_context: HashMap::new(),
                },
                max_length: Some(LengthConstraint {
                    limit: 255,
                    message: None,
                }),
            },
        }
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

    pub fn readonly(mut self, msg: &str) -> Self {
        self.config.base.readonly = Some(BoolChoice {
            choice: true,
            message: if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            },
        });
        self
    }

    pub fn max_length(mut self, limit: usize, msg: &str) -> Self {
        self.config.max_length = Some(LengthConstraint {
            limit,
            message: if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            },
        });
        self
    }
}

impl FormField for TextAreaField {
    fn name(&self) -> &str {
        &self.config.base.name
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
    fn field_type(&self) -> &str {
        &self.config.base.type_field
    }
    fn set_error(&mut self, message: String) {
        self.config.base.error = Some(message);
    }

    fn set_value(&mut self, value: &str) {
        self.config.base.value = value.to_string();
    }

    fn get_json_value(&self) -> Value {
        json!(self.config.base.value)
    }

    fn validate(&mut self) -> bool {
        let val = self.config.base.value.trim();

        if self.config.base.is_required.choice && val.is_empty() {
            let msg = self
                .config
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est requis".to_string());
            self.set_error(msg);
            return false;
        }

        if let Some(max_length) = &self.config.max_length {
            if val.chars().count() > max_length.limit {
                self.set_error(format!("Maximum {} caractères", max_length.limit));
                return false;
            }
        }

        self.config.base.error = None;
        true
    }

    fn set_name(&mut self, name: &str) {
        self.config.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.config.base.label = label.to_string();
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

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.config.base);
        context.insert("input_type", "textarea");

        tera.render(&self.config.base.template_name, &context)
            .map_err(|e| format!("Erreur rendu TextArea: {}", e))
    }
}
