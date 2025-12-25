use rusti::formulaire::forms_rusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, EmailField, IntegerField, PasswordField};
use fancy_regex::Regex;
use std::collections::HashMap;
use rusti::rusti_form;

#[rusti_form]
pub struct UserForm {
    pub form: Forms,
}

impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self { form: Forms::new() }
    }
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.is_valid_pur(raw_data)
    }
}

impl UserForm {

    pub fn is_valid_pur(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.require("username", &CharField { allow_blank: true }, raw_data);
        self.require("email", &EmailField, raw_data);
        self.require("age", &IntegerField, raw_data);

        // Password avec validation custom
        if let Some(raw_password) = raw_data.get("password") {
            self.validate_password(raw_password);
        } else {
            self.errors.insert("password".to_string(), "Requis".to_string());
        }

        self.is_valid()
    }

    fn validate_password(&mut self, raw_value: &str) -> Option<String> {
        let regex = Regex::new(
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&.\-_#^()+= \[\]{}|])[A-Za-z\d@$!%*?&.\-_#^()+= \[\]{}|]{10,}$"
        ).expect("Regex de mot de passe invalide");

        if raw_value.is_empty() {
            self.errors.insert("password".to_string(), "Le mot de passe est requis.".to_string());
            return None;
        }

        if !regex.is_match(raw_value).unwrap_or(false) {
            self.errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins 10 caractères, une majuscule, une minuscule, un chiffre et un caractère spécial.".to_string()
            );
            return None;
        }

        self.field("password", &PasswordField, raw_value)
    }
}