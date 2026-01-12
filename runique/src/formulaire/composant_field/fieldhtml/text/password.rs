use crate::formulaire::field::RuniqueField;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use serde_json::Value;
use tera::{Context, Tera};

pub struct PasswordField {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

impl PasswordField {
    pub fn new() -> Self {
        Self {
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: false,
        }
    }

    /// Configuration sécurisée pour la production
    pub fn secure() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }

    /// Configuration moyennement sécurisée
    pub fn moderate() -> Self {
        Self {
            min_length: 10,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
        }
    }

    /// Configuration basique (développement)
    pub fn basic() -> Self {
        Self {
            min_length: 6,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: false,
        }
    }

    pub fn with_min_length(mut self, length: usize) -> Self {
        self.min_length = length;
        self
    }

    pub fn require_uppercase(mut self) -> Self {
        self.require_uppercase = true;
        self
    }

    pub fn require_lowercase(mut self) -> Self {
        self.require_lowercase = true;
        self
    }

    pub fn require_digit(mut self) -> Self {
        self.require_digit = true;
        self
    }

    pub fn require_special(mut self) -> Self {
        self.require_special = true;
        self
    }

    /// Valide les règles de complexité
    fn validate_complexity(&self, password: &str) -> Result<(), String> {
        // Longueur minimale
        if password.len() < self.min_length {
            return Err(format!(
                "Le mot de passe doit contenir au moins {} caractères.",
                self.min_length
            ));
        }

        // Majuscule requise
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err("Le mot de passe doit contenir au moins une majuscule.".to_string());
        }

        // Minuscule requise
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err("Le mot de passe doit contenir au moins une minuscule.".to_string());
        }

        // Chiffre requis
        if self.require_digit && !password.chars().any(|c| c.is_numeric()) {
            return Err("Le mot de passe doit contenir au moins un chiffre.".to_string());
        }

        // Caractère spécial requis
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(
                "Le mot de passe doit contenir au moins un caractère spécial (!@#$%^&*...)."
                    .to_string(),
            );
        }

        Ok(())
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
        // Validation de complexité AVANT hachage
        self.validate_complexity(raw_value)?;

        // Hachage Argon2
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
        ctx.insert("value", ""); // Toujours vide pour sécurité
        ctx.insert("min_length", &self.min_length);

        // Ajouter les règles au contexte pour affichage dans le template
        let mut rules = Vec::new();
        rules.push(format!("Au moins {} caractères", self.min_length));
        if self.require_uppercase {
            rules.push("Une majuscule".to_string());
        }
        if self.require_lowercase {
            rules.push("Une minuscule".to_string());
        }
        if self.require_digit {
            rules.push("Un chiffre".to_string());
        }
        if self.require_special {
            rules.push("Un caractère spécial".to_string());
        }
        ctx.insert("rules", &rules);

        if let Some(err) = error {
            ctx.insert("error", err);
        }

        tera.render(self.template_name(), &ctx)
            .unwrap_or_else(|e| format!("Erreur Tera ({}): {}", self.template_name(), e))
    }
}
