use crate::formulaire::field::RuniqueField;
use chrono::NaiveDate;

pub struct DateField;

impl DateField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DateField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for DateField {
    type Output = NaiveDate;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        NaiveDate::parse_from_str(raw_value, "%Y-%m-%d")
            .map_err(|_| "Format de date invalide (AAAA-MM-JJ).".to_string())
    }

    fn template_name(&self) -> &str {
        "date"
    }
}