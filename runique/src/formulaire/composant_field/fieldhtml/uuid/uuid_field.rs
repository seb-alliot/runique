use crate::formulaire::field::RuniqueField;
use uuid::Uuid;

pub struct UUIDField;

impl RuniqueField for UUIDField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();

        match Uuid::parse_str(val) {
            Ok(uuid) => Ok(uuid.to_string()),
            Err(_) => Err("Format UUID invalide (attendu: 8-4-4-4-12 hex).".to_string()),
        }
    }

    fn template_name(&self) -> &str {
        "text"
    }
}
