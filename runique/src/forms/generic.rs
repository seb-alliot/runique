use crate::define_enum_kind;
use crate::delegate_to_kind;
use crate::forms::base::{CommonFieldConfig, FormField};
use crate::forms::fields::*;
use crate::utils::aliases::ATera;
use serde::Serialize;
use serde_json::Value;

define_enum_kind!(
    Text => TextField,
    Numeric => NumericField,
    #[serde(skip)]
    File => FileField,
    Boolean => BooleanField,
    Choice => ChoiceField,
    Radio => RadioField,
    Checkbox => CheckboxField,
    Date => DateField,
    Time => TimeField,
    DateTime => DateTimeField,
    Duration => DurationField,
    Color => ColorField,
    Slug => SlugField,
    UUID => UUIDField,
    JSON => JSONField,
    IPAddress => IPAddressField,
    Hidden => HiddenField,
);

#[derive(Clone, Serialize, Debug)]
pub struct GenericField {
    pub kind: FieldKind,
}

impl CommonFieldConfig for GenericField {
    fn get_field_config(&self) -> &crate::forms::base::FieldConfig {
        delegate_to_kind!(self, get_field_config)
    }

    fn get_field_config_mut(&mut self) -> &mut crate::forms::base::FieldConfig {
        delegate_to_kind!(mut self, get_field_config_mut)
    }
}

impl FormField for GenericField {
    // --- Getters ---

    fn name(&self) -> &str {
        delegate_to_kind!(self, name)
    }

    fn label(&self) -> &str {
        delegate_to_kind!(self, label)
    }

    fn value(&self) -> &str {
        delegate_to_kind!(self, value)
    }

    fn placeholder(&self) -> &str {
        delegate_to_kind!(self, placeholder)
    }

    fn field_type(&self) -> &str {
        delegate_to_kind!(self, field_type)
    }

    fn template_name(&self) -> &str {
        delegate_to_kind!(self, template_name)
    }

    fn required(&self) -> bool {
        delegate_to_kind!(self, required)
    }

    fn error(&self) -> Option<&String> {
        delegate_to_kind!(self, error)
    }

    // --- Setters ---

    fn set_name(&mut self, name: &str) {
        delegate_to_kind!(mut self, set_name, name)
    }

    fn set_label(&mut self, label: &str) {
        delegate_to_kind!(mut self, set_label, label)
    }

    fn set_value(&mut self, value: &str) {
        delegate_to_kind!(mut self, set_value, value)
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        delegate_to_kind!(mut self, set_placeholder, placeholder)
    }

    fn set_error(&mut self, error: String) {
        delegate_to_kind!(mut self, set_error, error)
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        delegate_to_kind!(mut self, set_html_attribute, key, value)
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        delegate_to_kind!(mut self, set_required, required, msg)
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        delegate_to_kind!(mut self, set_readonly, readonly, msg)
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        delegate_to_kind!(mut self, set_disabled, disabled, msg)
    }

    // --- Logique métier ---

    fn validate(&mut self) -> bool {
        delegate_to_kind!(mut self, validate)
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        delegate_to_kind!(self, render, tera)
    }

    // --- Sérialisation JSON ---

    fn to_json_value(&self) -> Value {
        delegate_to_kind!(self, to_json_value)
    }

    fn to_json_required(&self) -> Value {
        delegate_to_kind!(self, to_json_required)
    }

    fn to_json_attributes(&self) -> Value {
        delegate_to_kind!(self, to_json_attributes)
    }

    fn to_json_meta(&self) -> Value {
        delegate_to_kind!(self, to_json_meta)
    }
}
