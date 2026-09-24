//! `FormField` trait and `FieldConfig` structure: common base for all form fields.
use crate::forms::options::*;
use crate::utils::aliases::*;
use async_trait::async_trait;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Common configuration for a form field (name, label, value, error, HTML attributes).
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
    /// Field whose value must **never** leak out: not in the rendered widget,
    /// not in an audit trail, not in an admin view.
    ///
    /// Private, with no way to turn it back off: the only way to set it is
    /// [`FieldConfig::mark_password`] — there's no way to unset it. Read via
    /// [`FieldConfig::is_password`], which also returns `true` for any field of
    /// type `password` — so a sensitive field stays protected even if no one
    /// thought to set the flag.
    #[serde(default)]
    is_password: bool,
}

impl FieldConfig {
    /// Creates a config with the given name, semantic `type_field` (e.g. `"text"`,
    /// `"password"`), and Tera `template_name`. `is_password` is derived from
    /// `type_field` here, so a `"password"` type is always protected from the start.
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
            // Derived from the type right at construction: any `password` field,
            // whatever constructor was used, is born protected. The flag can
            // therefore never be missed by omission.
            is_password: type_field == "password",
        }
    }

    /// `true` if this field's value must never be exposed.
    ///
    /// The type wins over the flag: even if the field was built by hand without
    /// going through `mark_password`, a `type_field` of `password` is enough to
    /// protect it.
    #[must_use]
    pub fn is_password(&self) -> bool {
        self.is_password || self.type_field == "password"
    }

    /// Marks the field as carrying a secret (API key, token…).
    ///
    /// One-way, on purpose: there is no reverse operation. A field declared
    /// sensitive can't stop being sensitive along the way, which rules out any
    /// code path accidentally "unmasking" it.
    pub fn mark_password(&mut self) {
        self.is_password = true;
    }
}

/// Length constraints for text-like fields (min/max character length).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<LengthConstraint>,
    pub min_length: Option<LengthConstraint>,
}

/// Per-kind numeric validation state. Which variant a field holds depends on
/// its semantic (integer, float, decimal, percent, or a stepped range slider);
/// [`NumericField::min`](crate::forms::fields::NumericField::min) and `::max`
/// match on it to update the right bound representation.
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
    /// Bounded `0.0..=100.0` by default (see [`NumericField::percent`](crate::forms::fields::NumericField::percent)).
    Percent {
        value: Range,
    },
    /// Backs an `<input type="range">`; `default` is the initial value and
    /// `step` the increment.
    Range {
        value: Range,
        default: f64,
        step: f64,
    },
}

/// An inclusive `min..=max` bound used by [`NumericConfig`] variants.
#[derive(Clone, Serialize, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

/// Common trait to access a field's configuration
pub trait CommonFieldConfig {
    fn get_field_config(&self) -> &FieldConfig;
    fn get_field_config_mut(&mut self) -> &mut FieldConfig;

    /// `true` if the field's value must never be exposed.
    ///
    /// The framework's single point of truth on this: rendering, filling,
    /// logging and auditing all go through here instead of each comparing
    /// `field_type()` to the string `"password"` on their own.
    fn is_password(&self) -> bool {
        self.get_field_config().is_password()
    }
}

impl CommonFieldConfig for FieldConfig {
    fn get_field_config(&self) -> &FieldConfig {
        self
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        self
    }
}

/// Behavior every form field widget implements: validation, HTML rendering,
/// and JSON serialization for client-side use. Most getters/setters have
/// default implementations built on [`CommonFieldConfig`]; only
/// [`FormField::validate`] and [`FormField::render`] are field-type specific
/// and have no default.
#[async_trait]
pub trait FormField: CommonFieldConfig + DynClone + std::fmt::Debug + Send + Sync {
    // ========================================================================
    // GETTERS - Default implementation via CommonFieldConfig
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
    // SETTERS - Default implementation via CommonFieldConfig
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

    /// Resets the field error (explicit equivalent of `set_error("")`)
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

    /// Model-defined ceiling for max_size (file fields only). None for other field types.
    fn model_max_size(&self) -> Option<u64> {
        None
    }

    /// Overrides the effective max_size. Returns Err if it exceeds the model ceiling.
    fn set_max_size_bounded(
        &mut self,
        _size: crate::forms::fields::FileSize,
    ) -> Result<(), String> {
        Err("ce champ ne supporte pas max_size".to_string())
    }

    /// Field-type specific validation
    async fn validate(&mut self) -> bool;

    /// Rendering context shared by **every** field.
    ///
    /// Every [`FormField::render`] implementation starts from this and only adds
    /// its own variables. The `field_html/` templates all consume `field`,
    /// `readonly.choice` and `disabled.choice`: forgetting them was invisible
    /// under Tera 1, which evaluated a missing variable as false — the attribute
    /// was then simply never emitted, with no error at all. Nine renders out of
    /// eighteen were in that state.
    fn base_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        context.insert("field", self.get_field_config());
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());
        context
    }

    /// HTML rendering of the field
    fn render(&self, tera: &ATera) -> Result<String, String>;

    /// Finalization (e.g., password hashing)
    async fn finalize(&mut self) -> Result<(), String> {
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
