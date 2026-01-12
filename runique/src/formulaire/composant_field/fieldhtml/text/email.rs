use crate::formulaire::field::RuniqueField;
use fancy_regex::Regex;

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

        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if !re.is_match(&val).unwrap_or(false) {
            return Err("Format d'email invalide.".to_string());
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "email"
    }
}