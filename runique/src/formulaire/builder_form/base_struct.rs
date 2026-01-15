use crate::formulaire::builder_form::option_struct::*;
use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct FieldConfig {
    pub id: String,
    pub name: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_required: IsRequired,
    pub error: Option<String>,
    pub readonly: Option<ConfigReadOnly>,
    pub disabled: Option<ConfigDisabled>,
    pub type_field: String,
    pub html_attributes: HashMap<String, String>,
    pub template_name: String,
    pub extra_context: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct DateDisplayConfig {
    pub base: FieldConfig,
    pub display_value: String,
}


#[derive(Serialize)]
pub struct DateTimeConfig {
    pub base: FieldConfig,
    pub format: String,
    pub min_datetime: Option<String>,
    pub max_datetime: Option<String>,
}

#[derive(Serialize)]
pub struct TextConfig {
    pub base: FieldConfig,
    pub max_length: MaxlenghtConfig,
}

#[derive(Serialize)]
pub struct IntConfig {
    pub base: FieldConfig,
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Serialize)]
pub struct FloatConfig {
    pub base: FieldConfig,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Serialize)]
pub struct PositiveIntConfig {
    pub base: FieldConfig,
    pub max: Option<u64>,
}

#[derive(Serialize)]

pub struct BooleanConfig {
    pub base: FieldConfig,
    pub checked: bool,
}
