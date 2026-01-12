use crate::formulaire::field::RuniqueField;
use country::Country;
use postal_code::PostalCode;

pub struct PostalCodeField {
    pub country: Country,
}

impl PostalCodeField {
    pub fn new(country: Country) -> Self {
        Self { country }
    }
}

impl RuniqueField for PostalCodeField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();

        match PostalCode::new(self.country, val) {
            Ok(postal_code) => Ok(postal_code.to_string()),
            Err(_) => Err("Code postal invalide pour le pays sélectionné.".to_string()),
        }
    }

    fn template_name(&self) -> &str {
        "text"
    }
}
