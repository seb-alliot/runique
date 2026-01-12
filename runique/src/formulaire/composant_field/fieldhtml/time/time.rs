use crate::formulaire::field::RuniqueField;
use fancy_regex::Regex;

pub struct TimeField;

impl TimeField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimeField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for TimeField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Valide le format HH:MM ou HH:MM:SS
        let re = Regex::new(r"^([0-1][0-9]|2[0-3]):[0-5][0-9](:[0-5][0-9])?$").unwrap();

        if !re.is_match(raw_value).unwrap_or(false) {
            return Err("Format d'heure invalide (HH:MM ou HH:MM:SS).".to_string());
        }

        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "time"
    }
}