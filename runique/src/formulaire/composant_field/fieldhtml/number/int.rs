use crate::formulaire::field::RuniqueField;

pub struct IntegerField;

impl IntegerField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IntegerField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for IntegerField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .parse::<i64>()
            .map_err(|_| "Veuillez entrer un nombre entier.".to_string())
    }

    fn template_name(&self) -> &str {
        "number"
    }
}
