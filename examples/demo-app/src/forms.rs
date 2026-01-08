use runique::prelude::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct UsernameForm {
    #[serde(skip_deserializing)]
    pub form: Forms,
}

// Votre implémentation de Serialize est correcte pour envoyer vers Tera
impl Serialize for UsernameForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // On expose "form_html" pour le template Tera
        let mut state = serializer.serialize_struct("UsernameForm", 1)?;
        state.serialize_field("form_html", &self.form)?;
        state.end()
    }
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        // C'est ici que vous définissez le rendu pour votre format <div class="field-group">
        form.register_field("username", "Nom d'utilisateur", &CharField::new());
    }

    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
        // Validation type Django
        form.require("username", &CharField::new(), raw_data);
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }

    fn get_form(&self) -> &Forms {
        &self.form
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}
