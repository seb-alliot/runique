use crate::forms::options::*;
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
    pub type_field: String,
    pub html_attributes: HashMap<String, String>,
    pub template_name: String,
    pub extra_context: HashMap<String, String>,
}

impl FieldConfig {
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
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<LengthConstraint<usize>>,
    pub min_length: Option<LengthConstraint<usize>>,
    pub readonly: Option<BoolChoice>,
    pub disabled: Option<BoolChoice>,
}

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
    Percent {
        value: Range,
    },
    Range {
        value: Range,
        default: f64,
        step: f64,
    },
}

#[derive(Clone, Serialize, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}
