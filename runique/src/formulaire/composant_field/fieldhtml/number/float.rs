use crate::formulaire::field::RuniqueField;

pub struct FloatField;

impl FloatField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FloatField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for FloatField {
    type Output = f64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .replace(',', ".")
            .parse::<f64>()
            .map_err(|_| "Veuillez entrer un nombre décimal.".to_string())
    }

    fn template_name(&self) -> &str {
        "number"
    }
}
