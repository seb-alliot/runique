//! Selection field: `ChoiceField` with groups, multi-select, and dynamic options.
use crate::forms::base::{FieldConfig, FormField};
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::{Value, json};

/// Option for choice fields
#[derive(Clone, Debug, Serialize)]
pub struct ChoiceOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl ChoiceOption {
    /// Creates a new option with the given submitted value and display label.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            selected: false,
        }
    }

    /// Marks this option as pre-selected.
    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// Dropdown `<select>` field. Use [`ChoiceField::multiple`] for multi-select.
#[derive(Clone, Serialize, Debug)]
pub struct ChoiceField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
    pub multiple: bool,
}

impl ChoiceField {
    /// Creates an empty dropdown. Add options with [`choices`](ChoiceField::choices) or [`add_choice`](ChoiceField::add_choice).
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "select", "base_select.html"),
            choices: Vec::new(),
            multiple: false,
        }
    }

    /// Enables `<select multiple>`. Submitted value is a comma-separated string.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self.base.type_field = "select-multiple".to_string();
        self
    }

    /// Replaces all options at once.
    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    /// Appends a single option.
    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    /// Marks the field as required (empty selection fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

field_base!(ChoiceField, label);

#[async_trait]
impl FormField for ChoiceField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    /// A `multiple()` select sends one value per selected option, which reach
    /// the field joined as `"a,b"`: each one is marked selected for re-display.
    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
        if self.multiple {
            let selected: Vec<&str> = value.split(',').map(str::trim).collect();
            for choice in &mut self.choices {
                choice.selected = selected.contains(&choice.value.as_str());
            }
        }
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required("forms.select_required") {
            return false;
        }
        let val = self.base.value.trim();

        if !val.is_empty() && self.multiple {
            // Every selected value must be one of the choices, not the joined string.
            for selected in val.split(',').map(str::trim) {
                if !self.choices.iter().any(|c| c.value == selected) {
                    self.set_error(tf("forms.choice_invalid_value", &[selected]));
                    return false;
                }
            }
        } else if !val.is_empty() {
            // Check that the value exists in the choices
            let valid = self.choices.iter().any(|c| c.value == val);
            if !valid {
                self.set_error(t("forms.choice_invalid").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("choices", &self.choices);
        context.insert("multiple", &self.multiple);

        self.render_template(tera, &context)
    }
}

/// A group of `<input type="radio">` buttons where one option can be selected.
#[derive(Clone, Serialize, Debug)]
pub struct RadioField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
}

impl RadioField {
    /// Creates an empty radio group. Add options with [`choices`](RadioField::choices) or [`add_choice`](RadioField::add_choice).
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "radio", "base_radio.html"),
            choices: Vec::new(),
        }
    }

    /// Replaces all options at once.
    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    /// Appends a single option.
    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    /// Marks the field as required (no selection fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

field_base!(RadioField, label);

#[async_trait]
impl FormField for RadioField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required("forms.select_required") {
            return false;
        }
        let val = self.base.value.trim();

        if !val.is_empty() {
            let valid = self.choices.iter().any(|c| c.value == val);
            if !valid {
                self.set_error(t("forms.choice_invalid").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("choices", &self.choices);
        context.insert("meta", &self.to_json_meta());
        self.render_template(tera, &context)
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}

/// A group of checkboxes where multiple options can be selected simultaneously. Submitted value is a comma-separated string.
#[derive(Clone, Serialize, Debug)]
pub struct CheckboxField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
}

impl CheckboxField {
    /// Creates an empty checkbox group. Add options with [`choices`](CheckboxField::choices) or [`add_choice`](CheckboxField::add_choice).
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "checkbox", "base_checkbox.html"),
            choices: Vec::new(),
        }
    }

    /// Replaces all options at once.
    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    /// Appends a single option.
    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    /// Marks the field as required (no selection fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}
field_base!(CheckboxField, label);

#[async_trait]
impl FormField for CheckboxField {
    /// A list of choices lives in a table of its own, not in a column.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        Ok(None)
    }

    fn set_value(&mut self, value: &str) {
        // Expected format: "value1,value2,value3"
        self.base.value = value.to_string();
        let selected_values: Vec<&str> = value.split(',').map(|s| s.trim()).collect();

        for choice in &mut self.choices {
            choice.selected = selected_values.contains(&choice.value.as_str());
        }
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required("forms.checkbox_required") {
            return false;
        }
        let val = self.base.value.trim();

        if !val.is_empty() {
            // Check that all selected values exist
            let selected_values: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
            for sel_val in selected_values {
                if !self.choices.iter().any(|c| c.value == sel_val) {
                    self.set_error(tf("forms.choice_invalid_value", &[sel_val]));
                    return false;
                }
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("choices", &self.choices);
        context.insert("meta", &self.to_json_meta());

        self.render_template(tera, &context)
    }
}
