use crate::forms::options::*;
use crate::utils::aliases::ATera;
use crate::utils::aliases::*;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_required: BoolChoice,
    pub error: Option<String>,
    pub type_field: String,
    pub html_attributes: StrMap,
    pub template_name: String,
    pub extra_context: JsonMap,
}

impl FieldConfig {
    pub fn new(name: &str, type_field: &str, template_name: &str) -> Self {
        Self {
            name: name.to_string(),
            label: String::new(),
            value: String::new(),
            placeholder: String::new(),
            is_required: BoolChoice::default(),
            error: None,
            type_field: type_field.to_string(),
            html_attributes: HashMap::new(),
            template_name: template_name.to_string(),
            extra_context: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<LengthConstraint>,
    pub min_length: Option<LengthConstraint>,
}

#[derive(Clone, Serialize, Debug)]
pub enum NumericConfig {
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        value: Option<Range>,
    },
    Decimal {
        value: Option<Range>,
    },
    Percent {
        value: Range,
    },
    Range {
        value: Range,
        default: f64,
        step: f64,
    },
}

#[derive(Clone, Serialize, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

/// Trait commun pour accéder à la configuration d'un champ
pub trait CommonFieldConfig {
    fn get_field_config(&self) -> &FieldConfig;
    fn get_field_config_mut(&mut self) -> &mut FieldConfig;
}

impl CommonFieldConfig for FieldConfig {
    fn get_field_config(&self) -> &FieldConfig {
        self
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        self
    }
}

pub trait FormField: CommonFieldConfig + DynClone + std::fmt::Debug + Send + Sync {
    // ========================================================================
    // GETTERS - Implémentation par défaut via CommonFieldConfig
    // ========================================================================

    fn name(&self) -> &str {
        &self.get_field_config().name
    }

    fn label(&self) -> &str {
        &self.get_field_config().label
    }

    fn value(&self) -> &str {
        &self.get_field_config().value
    }

    fn placeholder(&self) -> &str {
        &self.get_field_config().placeholder
    }

    fn field_type(&self) -> &str {
        &self.get_field_config().type_field
    }

    fn template_name(&self) -> &str {
        &self.get_field_config().template_name
    }

    fn error(&self) -> Option<&String> {
        self.get_field_config().error.as_ref()
    }

    fn required(&self) -> bool {
        self.get_field_config().is_required.choice
    }

    // ========================================================================
    // SETTERS - Implémentation par défaut via CommonFieldConfig
    // ========================================================================

    fn set_name(&mut self, name: &str) {
        self.get_field_config_mut().name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.get_field_config_mut().label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.get_field_config_mut().value = value.to_string();
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        self.get_field_config_mut().placeholder = placeholder.to_string();
    }

    fn set_error(&mut self, message: String) {
        let config = self.get_field_config_mut();
        config.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    /// Réinitialise l'erreur du champ (équivalent explicite de set_error(""))
    fn clear_error(&mut self) {
        self.get_field_config_mut().error = None;
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.get_field_config_mut().is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.get_field_config_mut()
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        self.get_field_config_mut().extra_context.insert(
            "readonly".to_string(),
            json!({
                "choice": readonly,
                "message": msg.map(|s| s.to_string())
            }),
        );
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        self.get_field_config_mut().extra_context.insert(
            "disabled".to_string(),
            json!({
                "choice": disabled,
                "message": msg.map(|s| s.to_string())
            }),
        );
    }

    /// Validation spécifique au type de champ
    fn validate(&mut self) -> bool;

    /// Rendu HTML du champ
    fn render(&self, tera: &ATera) -> Result<String, String>;

    /// Finalisation (ex: hachage de mot de passe)
    fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn to_json_value(&self) -> Value {
        json!(self.get_field_config().value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.get_field_config().is_required)
    }

    fn to_json_readonly(&self) -> Value {
        self.get_field_config()
            .extra_context
            .get("readonly")
            .cloned()
            .unwrap_or_else(|| json!({"choice": false, "message": null}))
    }

    fn to_json_disabled(&self) -> Value {
        self.get_field_config()
            .extra_context
            .get("disabled")
            .cloned()
            .unwrap_or_else(|| json!({"choice": false, "message": null}))
    }

    fn to_json_attributes(&self) -> Value {
        let attrs: Vec<(&String, &String)> =
            self.get_field_config().html_attributes.iter().collect();
        let map: serde_json::Map<String, Value> = attrs
            .into_iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        Value::Object(map)
    }

    fn to_json_meta(&self) -> Value {
        json!({})
    }
}
