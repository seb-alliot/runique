use crate::formulaire::composant_field::utils::clean_slug_accent::remove_accents;
use crate::formulaire::field::RuniqueField;

pub struct SlugField;

impl SlugField {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SlugField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for SlugField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let slug = remove_accents(raw_value)
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if slug.is_empty() {
            return Err("Le titre ne peut pas être vide.".to_string());
        }

        Ok(slug)
    }
    fn template_name(&self) -> &str {
        "slug"
    }
}
