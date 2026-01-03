use runique::prelude::*;

#[derive(Deserialize)]
pub struct UsernameForm {
    pub form: Forms,
}

// Implémenter Serialize manuellement
impl Serialize for UsernameForm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("UsernameForm", 1)?;
        state.serialize_field("form_html", &self.form)?;
        state.end()
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
