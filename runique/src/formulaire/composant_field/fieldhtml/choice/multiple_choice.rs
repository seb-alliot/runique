use crate::formulaire::field::RuniqueField;
use crate::formulaire::field::SelectOption;

pub struct MultipleChoiceField {
    pub options: Vec<SelectOption>,
}

impl MultipleChoiceField {
    pub fn new(options: Vec<SelectOption>) -> Self {
        Self { options }
    }

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

impl RuniqueField for MultipleChoiceField {
    type Output = Vec<String>;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let selected: Vec<String> = raw_value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Vérifie que toutes les valeurs sélectionnées sont valides
        for val in &selected {
            if !self.options.iter().any(|o| &o.value == val) {
                return Err(format!("Option invalide: {}", val));
            }
        }

        if selected.is_empty() {
            return Err("Veuillez sélectionner au moins une option.".to_string());
        }

        Ok(selected)
    }

    fn template_name(&self) -> &str {
        "select-multiple"
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