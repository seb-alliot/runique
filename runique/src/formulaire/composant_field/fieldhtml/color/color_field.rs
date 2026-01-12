use crate::formulaire::field::RuniqueField;

pub struct ColorField;

impl RuniqueField for ColorField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();

        let re = fancy_regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();

        if !re.is_match(val).unwrap_or(false) {
            return Err("Format de couleur invalide (attendu: #RRGGBB).".to_string());
        }

        Ok(val.to_lowercase())
    }

    fn template_name(&self) -> &str {
        "color"
    }
}
