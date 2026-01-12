use crate::formulaire::field::RuniqueField;
use phonenumber::{country::Id as CountryId, parse, Mode};
use serde::Serialize;
use std::str::FromStr;

#[derive(Serialize, Default, Clone)]
pub struct PhoneField {
    pub default_country: Option<CountryId>,
    pub is_required: bool,
    pub allowed_countries: Option<Vec<CountryId>>,

    pub country_input: String,
    pub phone_input: String,

    pub error: Option<String>,
}

impl PhoneField {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    pub fn with_country(mut self, country: CountryId) -> Self {
        self.default_country = Some(country);
        self
    }

    pub fn fill(&mut self, country: &str, phone: &str) {
        self.country_input = country.trim().to_uppercase();
        self.phone_input = phone.trim().to_string();
    }

    pub fn validate(&mut self) -> Result<String, String> {
        if self.phone_input.is_empty() {
            let err = "Le numéro de téléphone est requis.".to_string();
            self.error = Some(err.clone());
        }

        let country_id = CountryId::from_str(&self.country_input)
            .ok()
            .or(self.default_country);

        if let Some(ref allowed) = self.allowed_countries {
            match country_id {
                Some(cid) if allowed.contains(&cid) => {}
                Some(_) => {
                    let err = "Ce pays n'est pas autorisé.".to_string();
                    self.error = Some(err.clone());
                    return Err(err);
                }
                None => {
                    let err = "Code pays requis.".to_string();
                    self.error = Some(err.clone());
                    return Err(err);
                }
            }
        }
        let phone = if let Some(cid) = country_id {
            parse(Some(cid), &self.phone_input)
        } else {
            parse(None, &self.phone_input)
        }
        .map_err(|_| {
            let err = "Format de numéro ou code pays invalide.".to_string();
            self.error = Some(err.clone());
            err
        })?;

        if !phone.is_valid() {
            let err = "Ce numéro n'est pas valide pour ce pays.".to_string();
            self.error = Some(err.clone());
            return Err(err);
        }

        self.error = None;
        Ok(phone.format().mode(Mode::E164).to_string())
    }
}

impl RuniqueField for PhoneField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let (country, phone) = raw_value.split_once('|').unwrap_or(("", raw_value));

        let mut tmp = self.clone();

        if tmp.default_country.is_none() {
            tmp.default_country = Some(CountryId::FR);
        }

        tmp.fill(country, phone);
        tmp.validate()
    }

    fn template_name(&self) -> &str {
        "telephone"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "country_value": self.country_input,
            "phone_value": self.phone_input,
            "required": self.is_required,
            "error": self.error,
            "default_country": self.default_country.map(|c| format!("{:?}", c))
        })
    }
}
