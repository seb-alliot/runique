use crate::formulaire::field::RuniqueField;
use chrono::NaiveDateTime;

pub struct DateTimeField;

impl DateTimeField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DateTimeField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for DateTimeField {
    type Output = NaiveDateTime;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est déjà trimmed
        let val = raw_value.replace('T', " ");

        NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M"))
            .map_err(|_| "Format date/heure invalide (AAAA-MM-JJ HH:MM:SS).".to_string())
    }

    fn template_name(&self) -> &str {
        "datetime-local"
    }
}
