use crate::formulaire::field::RuniqueField;


pub struct PositiveIntegerField;

impl PositiveIntegerField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PositiveIntegerField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for PositiveIntegerField {
    type Output = u64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: u64 = raw_value
            .parse()
            .map_err(|_| "Veuillez entrer un nombre entier positif.".to_string())?;

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "min": 0 })
    }
}