use crate::formulaire::field::RuniqueField;

pub struct DecimalField {
    pub max_digits: usize,
    pub decimal_places: usize,
}

impl DecimalField {
    pub fn new(max_digits: usize, decimal_places: usize) -> Self {
        Self {
            max_digits,
            decimal_places,
        }
    }
}

impl RuniqueField for DecimalField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.replace(',', ".");

        // Valide que c'est un nombre
        let parsed: f64 = val
            .parse()
            .map_err(|_| "Veuillez entrer un nombre décimal valide.".to_string())?;

        // Vérifie le nombre de décimales
        let parts: Vec<&str> = val.split('.').collect();
        if parts.len() > 1 && parts[1].len() > self.decimal_places {
            return Err(format!(
                "Maximum {} décimales autorisées.",
                self.decimal_places
            ));
        }

        // Vérifie le nombre total de chiffres
        let total_digits = val.replace(['.', '-'], "").len();
        if total_digits > self.max_digits {
            return Err(format!("Maximum {} chiffres autorisés.", self.max_digits));
        }

        Ok(format!("{:.prec$}", parsed, prec = self.decimal_places))
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "step": format!("0.{:0>width$}1", "", width = self.decimal_places.saturating_sub(1))
        })
    }
}