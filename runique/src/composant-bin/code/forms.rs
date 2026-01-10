use runique::prelude::*;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct UsernameForm {
    #[serde(skip_deserializing)]
    pub form: Forms,
}

impl Serialize for UsernameForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.form.serialize(serializer)
    }
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.register_field("username", "Nom d'utilisateur", &CharField::new());
    }

    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>) {
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
