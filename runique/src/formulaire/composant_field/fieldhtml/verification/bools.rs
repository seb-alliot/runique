use crate::formulaire::field::RuniqueField;

pub struct BooleanField;

impl BooleanField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BooleanField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for BooleanField {
    type Output = bool;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        match raw_value.to_lowercase().as_str() {
            "on" | "true" | "1" | "yes" | "checked" => Ok(true),
            _ => Ok(false),
        }
    }

    fn template_name(&self) -> &str {
        "checkbox"
    }
}