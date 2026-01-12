use crate::formulaire::field::RuniqueField;

pub struct URLField;

impl URLField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for URLField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for URLField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.starts_with("http") {
            Ok(raw_value.to_string())
        } else {
            Err("L'URL doit commencer par http:// ou https://".to_string())
        }
    }

    fn template_name(&self) -> &str {
        "url"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "type": "domaine" })
    }
}