use crate::formulaire::field::RuniqueField;

/// Champ radio pour booléen (Oui/Non)
pub struct BooleanRadioField {
    pub true_label: String,
    pub false_label: String,
}

impl BooleanRadioField {
    pub fn new() -> Self {
        Self {
            true_label: "Oui".to_string(),
            false_label: "Non".to_string(),
        }
    }

    pub fn with_labels(true_label: &str, false_label: &str) -> Self {
        Self {
            true_label: true_label.to_string(),
            false_label: false_label.to_string(),
        }
    }
}

impl Default for BooleanRadioField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for BooleanRadioField {
    type Output = bool;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        match raw_value {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err("Valeur invalide".to_string()),
        }
    }

    fn template_name(&self) -> &str {
        "radio"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "options": [
                {"value": "true", "label": self.true_label},
                {"value": "false", "label": self.false_label}
            ]
        })
    }
}
