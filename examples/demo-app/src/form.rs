use rusti::formulaire::forms_rusti::{Forms, FormulaireTrait};
use rusti::formulaire::field::{CharField, EmailField, IntegerField, PasswordField};
use serde::{Serialize, Deserialize};
use fancy_regex::Regex;
use std::collections::HashMap;
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserForm {
    #[serde(flatten)]
    pub internal: Forms,
}

impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self { internal: Forms::new() }
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.is_valid_pur(raw_data)
    }
}

impl UserForm {
    pub fn is_valid_pur(&mut self, raw_data: &HashMap<String, String>) -> bool {
        if let Some(raw_username) = raw_data.get("username") {
            self.internal.field("username", &CharField, raw_username);
        } else { self.internal.errors.insert("username".to_string(), "Requis".to_string()); }

        if let Some(raw_email) = raw_data.get("email") {
            self.internal.field("email", &EmailField, raw_email);
        } else { self.internal.errors.insert("email".to_string(), "Requis".to_string()); }

        if let Some(raw_age) = raw_data.get("age") {
            self.internal.field("age", &IntegerField, raw_age);
        } else { self.internal.errors.insert("age".to_string(), "Requis".to_string()); }

        if let Some(raw_password) = raw_data.get("password") {
            self.validate_password(raw_password);
        } else { self.internal.errors.insert("password".to_string(), "Requis".to_string()); }

        self.internal.is_valid()
    }

    fn validate_password(&mut self, raw_value: &str) -> Option<String> {
        let regex = Regex::new(
            r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&.\-_#^()+= \[\]{}|])[A-Za-z\d@$!%*?&.\-_#^()+= \[\]{}|]{10,}$"
        ).expect("Regex de mot de passe invalide");

        if raw_value.is_empty() {
            self.internal.errors.insert("password".to_string(), "Le mot de passe est requis.".to_string());
            return None;
        }

        if !regex.is_match(raw_value).unwrap_or(false) {
            self.internal.errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins 10 caractères, une majuscule, une minuscule, un chiffre et un caractère spécial.".to_string()
            );
            return None;
        }

        self.internal.field("password", &PasswordField, raw_value)
    }

    pub fn is_valid(&self) -> bool { self.internal.is_valid() }
    pub fn is_not_valid(&self) -> bool { !self.internal.is_valid() }

    pub fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(&self, field_name: &str) -> Option<T> {
        self.internal.get_value(field_name)
    }
}
