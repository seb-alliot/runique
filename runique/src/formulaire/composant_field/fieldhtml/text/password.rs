use crate::formulaire::field::RuniqueField;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use serde_json::Value;
use tera::{Context, Tera};

pub struct PasswordField;

impl PasswordField {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PasswordField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for PasswordField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.len() < 8 {
            return Err("Le mot de passe doit contenir au moins 8 caractères.".to_string());
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());

        let hashed = argon2
            .hash_password(raw_value.as_bytes(), &salt)
            .map_err(|_| "Erreur lors du hachage du mot de passe".to_string())?
            .to_string();

        Ok(hashed)
    }

    fn template_name(&self) -> &str {
        "password"
    }

    fn render(
        &self,
        tera: &Tera,
        name: &str,
        label: &str,
        _value: &Value,
        error: Option<&String>,
    ) -> String {
        let mut ctx = Context::new();
        ctx.insert("name", name);
        ctx.insert("label", label);
        ctx.insert("id", &format!("id_{}", name));
        ctx.insert("value", "");

        if let Some(err) = error {
            ctx.insert("error", err);
        }

        tera.render(self.template_name(), &ctx)
            .unwrap_or_else(|e| format!("Erreur Tera ({}): {}", self.template_name(), e))
    }
}