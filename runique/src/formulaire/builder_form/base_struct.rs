use crate::formulaire::builder_form::option_field::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
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

// On ajoute Default pour pouvoir initialiser GenericField facilement
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<LengthConstraint>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IntConfig {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FloatConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
}
