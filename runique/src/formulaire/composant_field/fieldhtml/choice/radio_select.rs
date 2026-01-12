use crate::formulaire::field::RuniqueField;

pub struct SelectOption {
    pub value: String,
    pub label: String,
}

pub struct RadioSelectField {
    pub options: Vec<SelectOption>,
}

impl RadioSelectField {
    pub fn new(options: Vec<SelectOption>) -> Self {
        Self { options }
    }

    /// Helper pour créer facilement des options depuis des tuples
    pub fn from_tuples(tuples: Vec<(&str, &str)>) -> Self {
        let options = tuples
            .into_iter()
            .map(|(value, label)| SelectOption {
                value: value.to_string(),
                label: label.to_string(),
            })
            .collect();

        Self { options }
    }
}

impl RuniqueField for RadioSelectField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Vérifie que la valeur existe dans les options
        if self.options.iter().any(|o| o.value == raw_value) {
            Ok(raw_value.to_string())
        } else {
            Err("Option invalide".to_string())
        }
    }

    fn template_name(&self) -> &str {
        "radio"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "options": self.options.iter().map(|o| {
                serde_json::json!({
                    "value": o.value,
                    "label": o.label
                })
            }).collect::<Vec<_>>()
        })
    }
}
