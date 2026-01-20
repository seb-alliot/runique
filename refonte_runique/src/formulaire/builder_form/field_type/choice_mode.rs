use crate::formulaire::builder_form::base_struct::FieldConfig;
use crate::formulaire::builder_form::option_field::BoolChoice;
use crate::formulaire::builder_form::trait_form::FormField;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tera::{Context, Tera};

/// Option pour les champs de choix
#[derive(Clone, Debug, Serialize)]
pub struct ChoiceOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

impl ChoiceOption {
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.to_string(),
            label: label.to_string(),
            selected: false,
        }
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// SelectField - Dropdown/Select standard
#[derive(Clone, Serialize, Debug)]
pub struct SelectField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
    pub multiple: bool,
}

impl SelectField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "select", "base_select"),
            choices: Vec::new(),
            multiple: false,
        }
    }

    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self.base.type_field = "select-multiple".to_string();
        self
    }

    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for SelectField {
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
        self.base.value = value.to_string();
        // Mettre à jour les options sélectionnées
        for choice in &mut self.choices {
            choice.selected = choice.value == value;
        }
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

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
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Veuillez sélectionner une option".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Vérifier que la valeur existe dans les choix
            let valid = self.choices.iter().any(|c| c.value == val);
            if !valid {
                self.set_error("Choix invalide".into());
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("choices", &self.choices);
        context.insert("multiple", &self.multiple);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
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

/// RadioField - Boutons radio
#[derive(Clone, Serialize, Debug)]
pub struct RadioField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
}

impl RadioField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "radio", "base_radio"),
            choices: Vec::new(),
        }
    }

    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for RadioField {
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
        self.base.value = value.to_string();
        for choice in &mut self.choices {
            choice.selected = choice.value == value;
        }
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
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Veuillez sélectionner une option".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            let valid = self.choices.iter().any(|c| c.value == val);
            if !valid {
                self.set_error("Choix invalide".into());
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("choices", &self.choices);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
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

/// CheckboxField - Checkboxes multiples
#[derive(Clone, Serialize, Debug)]
pub struct CheckboxField {
    pub base: FieldConfig,
    pub choices: Vec<ChoiceOption>,
}

impl CheckboxField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "checkbox", "base_checkbox"),
            choices: Vec::new(),
        }
    }

    pub fn choices(mut self, choices: Vec<ChoiceOption>) -> Self {
        self.choices = choices;
        self
    }

    pub fn add_choice(mut self, value: &str, label: &str) -> Self {
        self.choices.push(ChoiceOption::new(value, label));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for CheckboxField {
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
        // Format attendu: "value1,value2,value3"
        self.base.value = value.to_string();
        let selected_values: Vec<&str> = value.split(',').map(|s| s.trim()).collect();

        for choice in &mut self.choices {
            choice.selected = selected_values.contains(&choice.value.as_str());
        }
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
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Veuillez sélectionner au moins une option".into());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Vérifier que toutes les valeurs sélectionnées existent
            let selected_values: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
            for sel_val in selected_values {
                if !self.choices.iter().any(|c| c.value == sel_val) {
                    self.set_error(format!("Choix invalide: {}", sel_val));
                    return false;
                }
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("choices", &self.choices);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        let values: Vec<&str> = self
            .base
            .value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        json!(values)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}
