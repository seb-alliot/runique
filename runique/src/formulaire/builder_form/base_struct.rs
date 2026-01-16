use crate::formulaire::builder_form::option_field::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_required: BoolChoice,
    pub error: Option<String>,
    pub readonly: Option<BoolChoice>,
    pub disabled: Option<BoolChoice>,
    pub type_field: String,
    pub html_attributes: HashMap<String, String>,
    pub template_name: String,
    pub extra_context: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
pub struct DateDisplayConfig {
    pub base: FieldConfig,
    pub display_value: String,
}

#[derive(Serialize, Clone)]
pub struct DateTimeConfig {
    pub base: FieldConfig,
    pub format: String,
    pub min_datetime: Option<String>,
    pub max_datetime: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct TextConfig {
    pub base: FieldConfig,
    pub max_length: Option<LengthConstraint>,
}

#[derive(Serialize, Clone)]
pub struct IntConfig {
    pub base: FieldConfig,
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct FloatConfig {
    pub base: FieldConfig,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct PositiveIntConfig {
    pub base: FieldConfig,
    pub max: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct BooleanConfig {
    pub base: FieldConfig,
    pub checked: bool,
}
