use crate::formulaire::field::RuniqueField;
use fancy_regex::Regex;

pub struct DurationField;

impl DurationField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DurationField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for DurationField {
    type Output = u64; // Durée en secondes

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.to_lowercase();
        let mut total_seconds = 0u64;

        // Parse formats comme "2h30m", "90m", "1h", "3600s"
        let re = Regex::new(r"(\d+)([hms])").unwrap();

        for cap in re.captures_iter(&val) {
            if let Ok(capture) = cap {
                let num: u64 = capture[1].parse().unwrap_or(0);
                let unit = &capture[2];

                total_seconds += match unit {
                    "h" => num * 3600,
                    "m" => num * 60,
                    "s" => num,
                    _ => 0,
                };
            }
        }

        // Validation : s'assurer qu'au moins une durée a été trouvée
        if total_seconds == 0 {
            return Err("Durée invalide (formats acceptés: 2h30m, 90m, 1h, 3600s).".to_string());
        }

        Ok(total_seconds)
    }

    fn template_name(&self) -> &str {
        "text"
    }
}