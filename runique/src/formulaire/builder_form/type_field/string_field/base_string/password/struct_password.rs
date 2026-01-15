use crate::formulaire::validation_form::builder_form::trait_form::FormField;
use crate::formulaire::validation_form::builder_form::base_struct::FieldConfig;

use crate::app::TERA;
use tera::Context;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordField {
    pub base : FieldConfig,
    pub min_length: usize,
}

impl PasswordField {
    pub fn new(id: &str, name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                id: id.to_string(),
                name: name.to_string(),
                label: label.to_string(),
                value: "".to_string(),
                placeholder: "".to_string(),
                is_required: false,
                error: None,
            },
            min_length: 8,
        }
    }

    pub fn hash(&self) -> Result<String, String> {
        if self.base.value.is_empty() {
            return Err("Le mot de passe est vide".to_string());
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(self.base.value.as_bytes(), &salt) {
            Ok(h) => Ok(h.to_string()),
            Err(e) => Err(format!("Erreur de hachage : {}", e)),
        }
    }

    pub fn verify(password_plain: &str, password_hash: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password_plain.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

impl FormField for PasswordField {
    fn id(&self) -> &str { &self.base.id }
    fn name(&self) -> &str { &self.base.name }
    fn label(&self) -> &str { &self.base.label }
    fn value(&self) -> &str { &self.base.value }
    fn placeholder(&self) -> &str { &self.base.placeholder }
    fn get_error(&self) -> Option<&String> { self.base.error.as_ref() }

    fn validate(&mut self) -> bool {
        if self.base.is_required && self.base.value.is_empty() {
            self.base.error = Some("Le mot de passe est obligatoire".to_string());
            return false;
        }

        if self.base.value.chars().count() < self.min_length {
            self.base.error = Some(format!("Le mot de passe doit faire au moins {} caractères", self.min_length));
            return false;
        }

        self.base.error = None;
        true
    }
    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }
    fn render(&self) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("id", &self.base.id);
        context.insert("name", &self.base.name);
        context.insert("label", &self.base.label);
        context.insert("value", &self.base.value);
        context.insert("placeholder", &self.base.placeholder);
        context.insert("error", &self.base.error);
        context.insert("required", &self.base.is_required);
        context.insert("input_type", "password");

        TERA.render("base_string", &context)
            .map_err(|e| format!("Erreur rendu Password ({}): {}", self.base.id, e))
    }
}