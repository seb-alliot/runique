use crate::formulaire::field::RuniqueField;

pub struct CurrencyField {
    pub currency: String,
    pub decimal_places: usize,
}

impl CurrencyField {
    pub fn new(currency: &str) -> Self {
        Self {
            currency: currency.to_string(),
            decimal_places: 2,
        }
    }

    pub fn with_decimal_places(currency: &str, decimal_places: usize) -> Self {
        Self {
            currency: currency.to_string(),
            decimal_places,
        }
    }
}

impl Default for CurrencyField {
    fn default() -> Self {
        Self::new("€")
    }
}

impl RuniqueField for CurrencyField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let cleaned = raw_value
            .replace(&self.currency, "")
            .replace(' ', "")
            .replace(',', ".");

        let val: f64 = cleaned
            .parse()
            .map_err(|_| "Montant invalide.".to_string())?;

        if val < 0.0 {
            return Err("Le montant ne peut pas être négatif.".to_string());
        }

        // Formatte avec le nombre exact de décimales
        Ok(format!("{:.prec$}", val, prec = self.decimal_places))
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "step": format!("0.{:0>width$}1", "", width = self.decimal_places.saturating_sub(1)),
            "min": 0
        })
    }
}