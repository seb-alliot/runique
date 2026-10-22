//! Hidden field `HiddenField` for non-displayed data (tokens, internal IDs).
use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::aliases::ATera;
use crate::utils::constante::session_key::session::FORM_NAME_KEY;
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use serde::Serialize;
use subtle::ConstantTimeEq;

/// Hidden input field. Used internally for CSRF tokens; also available for opaque data
/// that should be submitted with the form but not displayed to the user.
#[derive(Clone, Serialize, Debug)]
pub struct HiddenField {
    pub base: FieldConfig,
    /// Expected session token (for CSRF validation)
    pub expected_value: Option<String>,
}

impl HiddenField {
    /// Specific constructor for a CSRF hidden field
    pub fn new_csrf() -> Self {
        Self {
            base: FieldConfig::new("csrf_token", "hidden", "csrf.html"),
            expected_value: None,
        }
    }

    /// Generic constructor for a hidden field
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "hidden", "base_hidden.html"),
            expected_value: None,
        }
    }

    /// Sets the expected value for validation (session token)
    pub fn set_expected_value(&mut self, expected: &str) {
        self.expected_value = Some(expected.to_string());
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }
}

impl CommonFieldConfig for HiddenField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

/// Hidden field carrying the form's name, set by `ValidationForm::try_form`.
/// Registered in every form; never filled from the request nor stored.
#[derive(Clone, Serialize, Debug)]
pub struct FormNameField {
    pub base: FieldConfig,
}

impl FormNameField {
    pub fn new() -> Self {
        Self {
            base: FieldConfig::new(FORM_NAME_KEY, "hidden", "base_form_name.html"),
        }
    }
}

impl Default for FormNameField {
    fn default() -> Self {
        Self::new()
    }
}

impl CommonFieldConfig for FormNameField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

#[async_trait]
impl FormField for FormNameField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        Ok(None)
    }

    async fn validate(&mut self) -> bool {
        true
    }

    /// An empty name leaves `field.value` out of the context: the template
    /// reads it unconditionally, so Tera fails the render instead of shipping
    /// a nameless form whose submissions would never validate.
    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut field = serde_json::Map::new();
        field.insert("name".to_string(), serde_json::json!(self.base.name));
        if !self.base.value.is_empty() {
            field.insert("value".to_string(), serde_json::json!(self.base.value));
        }
        let mut context = tera::Context::new();
        context.insert("field", &field);
        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

/// HoneypotField - invisible anti-bot trap field
#[derive(Clone, Serialize, Debug)]
pub struct HoneypotField {
    pub base: FieldConfig,
}

impl HoneypotField {
    /// Creates a honeypot field with the given name. Always validates as
    /// `true` — the anti-bot middleware, not the field itself, checks
    /// whether it was filled in and rejects the submission.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_honeypot.html"),
        }
    }
}

impl CommonFieldConfig for HoneypotField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

#[async_trait]
impl FormField for HoneypotField {
    /// A trap for bots: never stored.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        Ok(None)
    }

    async fn validate(&mut self) -> bool {
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let context = self.base_context();
        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

#[async_trait]
impl FormField for HiddenField {
    /// Stored only when given a column: the CSRF token, say, never is.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        if self.base.column_type.is_none() {
            return Ok(None);
        }
        crate::forms::column_value::read_field(
            &self.base,
            &self.base.value,
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    async fn validate(&mut self) -> bool {
        // For a CSRF field, check that the value matches the expected one
        if self.base.name == "csrf_token"
            && let Some(expected) = &self.expected_value
        {
            if self.base.value.trim().is_empty() {
                self.set_error(t("csrf.missing").to_string());
                return false;
            }

            // ct_eq: constant-time comparison — prevents an attacker
            // guessing the token byte by byte via response time
            if !bool::from(self.base.value.as_bytes().ct_eq(expected.as_bytes())) {
                self.set_error(t("csrf.invalid").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("input_type", &self.base.type_field);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}
