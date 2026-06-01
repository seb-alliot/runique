//! Boolean field `BooleanField`: HTML checkbox with unchecked case management.
use crate::forms::base::*;
use crate::utils::trad::tf;
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};

/// Checkbox or radio input for boolean values.
/// Use [`BooleanField::new`] for a checkbox, [`::radio`](BooleanField::radio) for a radio button.
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
    /// Creates a checkbox field. Unchecked state is normalized to `"false"` by `fill()`.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "checkbox", "base_boolean.html"),
        }
    }

    /// Creates a radio button field (`<input type="radio">`).
    pub fn radio(name: &str) -> Self {
        let mut field = Self::new(name);
        field.base.type_field = "radio".to_string();
        field
    }

    /// Marks the field as required. For a checkbox, this means NOT NULL in DB — not "must be checked".
    /// To force the user to tick (e.g. ToS), use `clean()` with a custom error instead.
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Pre-selects the checkbox as checked.
    pub fn checked(mut self) -> Self {
        self.base.value = "true".to_string();
        self
    }

    /// Pre-selects the checkbox as unchecked (default).
    pub fn unchecked(mut self) -> Self {
        self.base.value = "false".to_string();
        self
    }
}

impl FormField for BooleanField {
    fn validate(&mut self) -> bool {
        // A boolean field is always valid: "true" or "false" (unchecked = false).
        // required = NOT NULL in DB, not "must be checked".
        // To force the check (e.g., TOS), use clean() with a custom error.
        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("input_type", &self.base.type_field);
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());

        // Add the "checked" state
        let is_checked = self.base.value == "true";
        context.insert("checked", &is_checked);

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
