use crate::formulaire::field::RuniqueField;

pub struct PercentageField;

impl PercentageField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PercentageField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for PercentageField {
    type Output = f64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: f64 = raw_value
            .replace(',', ".")
            .parse()
            .map_err(|_| "Veuillez entrer un nombre.".to_string())?;

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "step": 0.01
        })
    }
}