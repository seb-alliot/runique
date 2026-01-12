use crate::formulaire::field::RuniqueField;

pub struct RangeField {
    pub min: i64,
    pub max: i64,
    pub step: i64,
}

impl RangeField {
    pub fn new(min: i64, max: i64) -> Self {
        Self {
            min,
            max,
            step: 1
        }
    }

    pub fn with_step(min: i64, max: i64, step: i64) -> Self {
        Self { min, max, step }
    }
}

impl RuniqueField for RangeField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: i64 = raw_value
            .parse()
            .map_err(|_| "Valeur numérique invalide.".to_string())?;

        if val < self.min || val > self.max {
            return Err(format!("La valeur doit être comprise entre {} et {}.", self.min, self.max));
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "range"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "min": self.min,
            "max": self.max,
            "step": self.step
        })
    }
}