use rusti::formulaire::forms::{Forms, FormulaireTrait}; // Ajoute l'import du Trait
use rusti::formulaire::field::{CharField, EmailField, IntegerField, PasswordField};
use serde::{Deserialize, Serialize};
use fancy_regex::Regex;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserForm(Forms);

impl FormulaireTrait for UserForm {
    fn new() -> Self {
        Self(Forms::new())
    }

    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool {
        self.is_valid_pur(raw_data)
    }
}

impl UserForm {
    pub fn is_valid_pur(&mut self, raw_data: &HashMap<String, String>) -> bool {
        if let Some(raw_username) = raw_data.get("username") {
            self.0.field("username", &CharField, raw_username);
        } else {
            self.0.errors.insert("username".to_string(), "Le nom d'utilisateur est requis.".to_string());
        }

        if let Some(raw_email) = raw_data.get("email") {
            self.0.field("email", &EmailField, raw_email);
        } else {
            self.0.errors.insert("email".to_string(), "L'email est requis.".to_string());
        }

        if let Some(raw_age) = raw_data.get("age") {
            self.0.field("age", &IntegerField, raw_age);
        } else {
            self.0.errors.insert("age".to_string(), "L'âge est requis.".to_string());
        }

        if let Some(raw_password) = raw_data.get("password") {
            self.validate_password(raw_password);
        } else {
            self.0.errors.insert("password".to_string(), "Le mot de passe est requis.".to_string());
        }

        self.0.is_valid()
    }

    fn validate_password(&mut self, raw_value: &str) -> Option<String> {
        let regex = Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&.\-_#^()+= [\]{}|])[A-Za-z\d@$!%*?&.\-_#^()+= [\]{}|]{10,}$").unwrap();

        if raw_value.is_empty() {
            self.0.errors.insert("password".to_string(), "Le mot de passe est requis.".to_string());
            return None;
        }

        if !regex.is_match(raw_value).unwrap_or(false) {
            self.0.errors.insert(
                "password".to_string(),
                "Le mot de passe doit contenir au moins 10 caractères, une majuscule, une minuscule, un chiffre et un caractère spécial.".to_string()
            );
            return None;
        }

        self.0.field("password", &PasswordField, raw_value)
    }

    pub fn is_valid(&self) -> bool { self.0.is_valid() }
}