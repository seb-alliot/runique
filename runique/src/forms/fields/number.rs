use crate::forms::base::*;
use crate::forms::field::FormField;
use crate::forms::options::BoolChoice;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone, Serialize, Debug)]
pub struct NumericField {
    pub base: FieldConfig,
    pub config: NumericConfig,
    pub min_digits: Option<usize>,
    pub max_digits: Option<usize>,
}

impl NumericField {
    fn create(name: &str, type_field: &str, config: NumericConfig) -> Self {
        Self {
            base: FieldConfig::new(name, type_field, "base_number"),
            config,
            min_digits: None,
            max_digits: None,
        }
    }
    pub fn digits(mut self, min: usize, max: usize) -> Self {
        self.min_digits = Some(min);
        self.max_digits = Some(max);
        self
    }
    // --- Constructeurs ---
    pub fn integer(name: &str) -> Self {
        Self::create(
            name,
            "number",
            NumericConfig::Integer {
                min: None,
                max: None,
            },
        )
    }

    pub fn float(name: &str) -> Self {
        Self::create(name, "number", NumericConfig::Float { value: None })
    }

    pub fn decimal(name: &str) -> Self {
        Self::create(name, "number", NumericConfig::Decimal { value: None })
    }

    pub fn percent(name: &str) -> Self {
        Self::create(
            name,
            "number",
            NumericConfig::Percent {
                value: Range {
                    min: 0.0,
                    max: 100.0,
                },
            },
        )
    }

    pub fn range(name: &str, min: f64, max: f64, default: f64) -> Self {
        let mut field = Self::create(
            name,
            "range",
            NumericConfig::Range {
                value: Range { min, max },
                default,
                step: 1.0,
            },
        );
        field.base.value = default.to_string();
        field
    }

    // --- Builder Methods ---
    pub fn min(mut self, val: f64, msg: &str) -> Self {
        match &mut self.config {
            NumericConfig::Integer { min, .. } => *min = Some(val as i64),
            NumericConfig::Float { value } | NumericConfig::Decimal { value, .. } => {
                if let Some(f) = value {
                    f.min = val;
                } else {
                    *value = Some(Range {
                        min: val,
                        max: f64::MAX,
                    });
                }
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                value.min = val;
            }
        }
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn max(mut self, val: f64, msg: &str) -> Self {
        match &mut self.config {
            NumericConfig::Integer { max, .. } => *max = Some(val as i64),
            NumericConfig::Float { value } | NumericConfig::Decimal { value, .. } => {
                if let Some(f) = value {
                    f.max = val;
                } else {
                    *value = Some(Range {
                        min: f64::MIN,
                        max: val,
                    });
                }
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                value.max = val;
            }
        }
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn step(mut self, s: f64) -> Self {
        if let NumericConfig::Range { step, .. } = &mut self.config {
            *step = s;
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }
}

// --- Implémentation du Trait ---
impl FormField for NumericField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn template_name(&self) -> &str {
        &self.base.template_name
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
    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_error(&mut self, error: String) {
        self.base.error = if error.is_empty() { None } else { Some(error) };
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
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
            self.set_error(
                self.base
                    .is_required
                    .message
                    .clone()
                    .unwrap_or_else(|| "Requis".into()),
            );
            return false;
        }
        if val.is_empty() {
            return true;
        }

        let normalized = val.replace(',', ".");

        // --- ÉTAPE 1 : Validation de la précision (digits) ---
        let current_digits = normalized
            .find('.')
            .map(|dot| normalized[dot + 1..].len())
            .unwrap_or(0);

        if current_digits < self.min_digits.unwrap_or(0) {
            self.set_error(format!(
                "Il faut au moins {} chiffres après la virgule",
                self.min_digits.unwrap_or(0)
            ));
            return false;
        }
        if current_digits > self.max_digits.unwrap_or(usize::MAX) {
            self.set_error(format!(
                "Maximum {} chiffres après la virgule autorisés",
                self.max_digits.unwrap_or(usize::MAX)
            ));
            return false;
        }

        // --- ÉTAPE 2 : Validation des bornes de valeur (min/max) ---
        match &self.config {
            NumericConfig::Integer { min, max } => {
                if let Ok(v) = normalized.parse::<i64>() {
                    if let Some(m) = min {
                        if v < *m {
                            self.set_error(format!("Minimum: {}", m));
                            return false;
                        }
                    }
                    if let Some(m) = max {
                        if v > *m {
                            self.set_error(format!("Maximum: {}", m));
                            return false;
                        }
                    }
                } else {
                    self.set_error("Nombre entier invalide".into());
                    return false;
                }
            }
            NumericConfig::Decimal { value, .. } | NumericConfig::Float { value } => {
                if let Ok(v) = normalized.parse::<f64>() {
                    if let Some(f) = value.as_ref() {
                        if v < f.min {
                            self.set_error(format!("Trop bas (min: {})", f.min));
                            return false;
                        }
                        if v > f.max {
                            self.set_error(format!("Trop haut (max: {})", f.max));
                            return false;
                        }
                    }
                } else {
                    self.set_error("Nombre invalide".into());
                    return false;
                }
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                match normalized.parse::<f64>() {
                    Ok(v) => {
                        if v < value.min || v > value.max {
                            self.set_error("Valeur incorrecte".into());
                            return false;
                        }
                    }
                    Err(_) => {
                        self.set_error("Invalide".into());
                        return false;
                    }
                }
            }
        }
        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("config", &self.config);
        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        match &self.config {
            NumericConfig::Integer { .. } => self
                .base
                .value
                .parse::<i64>()
                .map(|v| json!(v))
                .unwrap_or(json!(null)),
            _ => self
                .base
                .value
                .replace(',', ".")
                .parse::<f64>()
                .map(|v| json!(v))
                .unwrap_or(json!(null)),
        }
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }
    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}
