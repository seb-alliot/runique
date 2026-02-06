use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField, TextConfig};
pub use crate::forms::generic::GenericField;
use crate::forms::options::LengthConstraint;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};
use validator::{ValidateEmail, ValidateUrl};

// Structure principale pour les champs texte
#[derive(Clone, Serialize, Debug)]
pub struct TextField {
    pub base: FieldConfig,
    pub config: TextConfig,
    pub format: SpecialFormat,
}

// Formats spéciaux pour les champs texte
#[derive(Clone, Debug, Serialize)]
pub enum SpecialFormat {
    None,
    Email,
    Url,
    Password,
    RichText,
    Csrf,
}
impl CommonFieldConfig for TextField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

// Implémentation des méthodes pour TextField
impl TextField {
    // Constructeur privé général basé sur le field générique => évite la duplication de code
    fn create(name: &str, type_field: &str, format: SpecialFormat) -> Self {
        Self {
            base: FieldConfig::new(name, type_field, "base_string"),
            config: TextConfig::default(),
            format,
        }
    }
    pub fn create_csrf() -> Self {
        let mut field = Self::create("csrf_token", "hidden", SpecialFormat::Csrf);
        field.base.template_name = "csrf".to_string();
        field
    }

    pub fn min_length(mut self, min: usize, msg: &str) -> Self {
        self.config.min_length = Some(LengthConstraint {
            value: min,
            message: (!msg.is_empty()).then(|| msg.to_string()),
        });
        self
    }

    pub fn max_length(mut self, max: usize, msg: &str) -> Self {
        self.config.max_length = Some(LengthConstraint {
            value: max,
            message: (!msg.is_empty()).then(|| msg.to_string()),
        });
        self
    }

    // Constructeurs publics pour différents types de champs texte
    pub fn text(name: &str) -> Self {
        Self::create(name, "text", SpecialFormat::None)
    }
    pub fn textarea(name: &str) -> Self {
        Self::create(name, "textarea", SpecialFormat::None)
    }
    pub fn richtext(name: &str) -> Self {
        Self::create(name, "richtext", SpecialFormat::RichText)
    }
    pub fn password(name: &str) -> Self {
        Self::create(name, "password", SpecialFormat::Password)
    }
    pub fn email(name: &str) -> Self {
        let mut field = Self::create(name, "email", SpecialFormat::Email);
        field.base.value = field.base.value.to_lowercase();
        field
    }
    pub fn url(name: &str) -> Self {
        Self::create(name, "url", SpecialFormat::Url)
    }

    // Utilitaires mot de passe
    pub fn hash_password(&self) -> Result<String, String> {
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

    pub fn verify_password(password_plain: &str, password_hash: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password_plain.as_bytes(), &parsed_hash)
            .is_ok()
    }

    // Builder methods
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    pub fn readonly(mut self, msg: &str) -> Self {
        self.set_readonly(true, Some(msg));
        self
    }

    pub fn disabled(mut self, msg: &str) -> Self {
        self.set_disabled(true, Some(msg));
        self
    }
}

impl FormField for TextField {
    fn validate(&mut self) -> bool {
        // Trim initial
        let mut val = self.base.value.trim().to_string();

        if let SpecialFormat::RichText = self.format {
            val = crate::utils::sanitizer::sanitize(&self.base.name, &val);
        }

        // Validation du champ requis
        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est obligatoire".into());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            return true;
        }

        // Validation longueur min
        if let Some(limits) = &self.config.min_length {
            let count = val.chars().count();
            if count < limits.value {
                let msg = limits
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Trop court (min {})", limits.value));
                self.set_error(msg);
                return false;
            }
        }

        // Validation longueur max
        if let Some(limits) = &self.config.max_length {
            let count = val.chars().count();
            if count > limits.value {
                let msg = limits
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("Trop long (max {})", limits.value));
                self.set_error(msg);
                return false;
            }
        }

        // Validation format spécial
        match &self.format {
            SpecialFormat::Email if !val.validate_email() => {
                self.set_error("Format d'adresse email invalide".into());
                return false;
            }
            SpecialFormat::Email => {
                val = val.to_lowercase();
            }
            SpecialFormat::Url if !val.validate_url() => {
                self.set_error("Veuillez entrer une URL valide".into());
                return false;
            }
            _ => {}
        }

        // Mise à jour la valeur nettoyée
        self.base.value = val;

        self.clear_error();
        true
    }

    fn finalize(&mut self) -> Result<(), String> {
        if let SpecialFormat::Password = &self.format {
            // On ne hache que si ce n'est pas déjà fait
            if !self.base.value.is_empty() && !self.base.value.starts_with("$argon2") {
                match self.hash_password() {
                    Ok(h) => self.base.value = h,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }
    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();

        // On prépare une version "sécurisée" de la base
        let mut base_data = self.base.clone();
        if let SpecialFormat::Password = &self.format {
            base_data.value = "".to_string();
        }

        context.insert("field", &base_data);
        context.insert("input_type", &self.base.type_field);

        // AJOUT IMPORTANT : On injecte la config pour readonly/disabled
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());

        if let Some(l) = &self.config.min_length {
            context.insert("min_length", &l.value);
        }
        if let Some(l) = &self.config.max_length {
            context.insert("max_length", &l.value);
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}
