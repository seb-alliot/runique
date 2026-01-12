use crate::formulaire::field::RuniqueField;
use crate::formulaire::sanetizer;

pub struct TextField {
    pub allow_blank: bool,
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl TextField {
    pub fn new() -> Self {
        Self { allow_blank: false }
    }

    pub fn allow_blank() -> Self {
        Self { allow_blank: true }
    }
}

impl RuniqueField for TextField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas être vide".to_string());
        }

        Ok(sanetizer::auto_sanitize(raw_value))
    }

    fn template_name(&self) -> &str {
        "textarea"
    }
}