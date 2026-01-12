use crate::formulaire::field::RuniqueField;
use validator::ValidateEmail;

pub struct EmailField;

impl EmailField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmailField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for EmailField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim().to_lowercase();

        if !val.validate_email() {
            return Err("Format d'email invalide.".to_string());
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "email"
    }
}
