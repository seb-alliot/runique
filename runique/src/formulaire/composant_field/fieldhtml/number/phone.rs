use crate::formulaire::field::RuniqueField;

pub struct PhoneField;

impl PhoneField {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PhoneField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for PhoneField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let cleaned: String = raw_value
            .chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect();

        if cleaned.contains('+') && !cleaned.starts_with('+') {
            return Err("Le signe '+' doit être au début du numéro.".to_string());
        }

        let digit_count = cleaned.chars().filter(|c| c.is_numeric()).count();
        if digit_count < 8 || digit_count > 15 {
            return Err("Le numéro doit contenir entre 8 et 15 chiffres.".to_string());
        }

        Ok(cleaned)
    }

    fn template_name(&self) -> &str {
        "tel"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "tel",
            "placeholder": "ex: +33 6 12 34 56 78"
        })
    }
}