use crate::formulaire::field::RuniqueField;
use serde_json::Value;

pub struct JSONField;

impl JSONField {
    pub fn new() -> Self {
        Self
    }
}
impl Default for JSONField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for JSONField {
    type Output = Value;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        serde_json::from_str(raw_value).map_err(|_| "Contenu JSON malformé.".to_string())
    }
    fn template_name(&self) -> &str {
        "json"
    }
}
