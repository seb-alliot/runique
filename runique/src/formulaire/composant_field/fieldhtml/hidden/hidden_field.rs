use crate::formulaire::field::RuniqueField;

pub struct HiddenField;

impl RuniqueField for HiddenField {
    type Output = String;
    fn process(&self, v: &str) -> Result<Self::Output, String> {
        Ok(v.to_string())
    }

    fn template_name(&self) -> &str {
        "hidden"
    }
}