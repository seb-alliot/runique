//! Boolean field `BooleanField`: HTML checkbox with unchecked case management.
use crate::forms::base::*;
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use serde::Serialize;

/// Checkbox or radio input for boolean values.
/// Use [`BooleanField::new`] for a checkbox, [`::radio`](BooleanField::radio) for a radio button.
#[derive(Clone, Serialize, Debug)]
pub struct BooleanField {
    pub base: FieldConfig,
    /// What the box sends when checked, and the only value read as `true`.
    pub checked_value: String,
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
            checked_value: "true".to_string(),
        }
    }

    /// The value the box sends when checked (`"true"` by default) — for a
    /// hand-written template whose box sends something else, `"on"` for one
    /// with no `value` attribute.
    pub fn checked_value(mut self, value: &str) -> Self {
        self.checked_value = value.to_string();
        self
    }

    /// The box's state: `Ok(true)` checked, `Ok(false)` unchecked (`fill()`
    /// turns an absent box into "false"), `Err` for anything the rendered box
    /// can't send — a forged request.
    fn state(&self) -> Result<bool, ()> {
        match self.base.value.as_str() {
            "" | "false" => Ok(false),
            value if value == self.checked_value => Ok(true),
            _ => Err(()),
        }
    }

    /// Creates a radio button field (`<input type="radio">`).
    pub fn radio(name: &str) -> Self {
        let mut field = Self::new(name);
        field.base.type_field = "radio".to_string();
        field
    }

    /// The box must be checked (terms, consent): an unchecked box is an error.
    /// A box always answers, so NOT NULL needs nothing; this is the only
    /// meaning `required` has on a checkbox.
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
        self.base.value = self.checked_value.clone();
        self
    }

    /// Pre-selects the checkbox as unchecked (default).
    pub fn unchecked(mut self) -> Self {
        self.base.value = "false".to_string();
        self
    }
}

#[async_trait]
impl FormField for BooleanField {
    /// Unchecked (or never filled) is `false`, never empty.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        self.state()
            .map(|checked| Some(sea_orm::Value::from(checked)))
            .map_err(|()| t("forms.value_unconvertible").into_owned())
    }

    async fn validate(&mut self) -> bool {
        let refusal = match self.state() {
            // The rendered box only sends its own value: anything else was forged.
            Err(()) => Some((
                t("forms.value_unconvertible").into_owned(),
                "boolean refused: not the value the box sends",
            )),
            Ok(false) if self.base.is_required.choice => Some((
                self.base
                    .is_required
                    .message
                    .clone()
                    .unwrap_or_else(|| t("forms.must_be_checked").into_owned()),
                "required box left unchecked",
            )),
            Ok(_) => None,
        };
        let Some((message, reason)) = refusal else {
            self.clear_error();
            return true;
        };
        if let Some(level) = crate::utils::runique_log::get_log()
            .forms
            .as_ref()
            .and_then(|f| f.validate)
        {
            crate::runique_log!(level, field = %self.base.name, reason, "boolean field refused");
        }
        self.set_error(message);
        false
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("input_type", &self.base.type_field);

        // Add the "checked" state
        context.insert("checked", &self.state().unwrap_or(false));
        context.insert("checked_value", &self.checked_value);

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
