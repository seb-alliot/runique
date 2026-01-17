use crate::formulaire::builder_form::base_struct::*;
use crate::formulaire::builder_form::option_field::{BoolChoice, LengthConstraint};
use crate::formulaire::builder_form::trait_form::FormField;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Serialize;

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};
use validator::{ValidateEmail, ValidateUrl};

#[derive(Clone, Debug, Serialize)]
pub enum SpecialFormat {
    None,
    Email,
    Url,
    Password,
    RichText,
}

#[derive(Clone, Debug, Serialize)]
pub enum FieldKind {
    Text {
        config: TextConfig,
        format: SpecialFormat,
    },
    Integer(IntConfig),
    Float(FloatConfig),
    Boolean,
}

#[derive(Clone, Serialize)]
pub struct GenericField {
    pub base: FieldConfig,
    pub kind: FieldKind,
}

impl GenericField {
    // Constructeur de base
    fn create(name: &str, type_field: &str, kind: FieldKind) -> Self {
        Self {
            base: FieldConfig {
                name: name.to_string(),
                label: String::new(),
                value: String::new(),
                placeholder: String::new(),
                is_required: BoolChoice::default(),
                min_length: None,
                max_length: None,
                error: None,
                readonly: None,
                disabled: None,
                type_field: type_field.to_string(),
                html_attributes: HashMap::new(),
                template_name: "base_string".to_string(),
                extra_context: HashMap::new(),
            },
            kind,
        }
    }

    // Constructeurs de champs
    pub fn text(name: &str) -> Self {
        Self::create(
            name,
            "text",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::None,
            },
        )
    }

    pub fn textarea(name: &str) -> Self {
        let mut f = Self::create(
            name,
            "textarea",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::None,
            },
        );
        f.base.template_name = "base_string".to_string();
        f
    }

    pub fn richtext(name: &str) -> Self {
        let mut f = Self::create(
            name,
            "richtext",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::RichText,
            },
        );
        f.base.template_name = "base_string".to_string();
        f
    }

    pub fn password(name: &str) -> Self {
        Self::create(
            name,
            "password",
            FieldKind::Text {
                config: TextConfig {
                    max_length: None,
                    min_length: None,
                },
                format: SpecialFormat::Password,
            },
        )
    }

    pub fn email(name: &str) -> Self {
        let mut f = Self::create(
            name,
            "email",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::Email,
            },
        );
        f.base.placeholder = "exemple@domaine.com".to_string();
        f
    }

    pub fn url(name: &str) -> Self {
        Self::create(
            name,
            "url",
            FieldKind::Text {
                config: TextConfig {
                    max_length: None,
                    min_length: None,
                },
                format: SpecialFormat::Url,
            },
        )
    }

    pub fn int(name: &str) -> Self {
        Self::create(name, "number", FieldKind::Integer(IntConfig::default()))
    }

    // Utilitaires spécifiques au mot de passe
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
    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    pub fn min_length(mut self, min: usize, msg: &str) -> Self {
        if let FieldKind::Text { ref mut config, .. } = self.kind {
            let current_max = config.min_length.as_ref().and_then(|l| l.max);
            config.min_length = Some(LengthConstraint {
                min: Some(min),
                max: current_max,
                message: if msg.is_empty() {
                    None
                } else {
                    Some(msg.to_string())
                },
            });
        }
        self
    }

    pub fn max_length(mut self, max: usize, msg: &str) -> Self {
        if let FieldKind::Text { ref mut config, .. } = self.kind {
            let current_min = config.max_length.as_ref().and_then(|l| l.min);
            config.max_length = Some(LengthConstraint {
                min: current_min,
                max: Some(max),
                message: if msg.is_empty() {
                    None
                } else {
                    Some(msg.to_string())
                },
            });
        }
        self
    }
}

// --- IMPLÉMENTATION DU TRAIT FormField ---
impl FormField for GenericField {
    // Getters
    fn name(&self) -> &str {
        &self.base.name
    }

    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> &str {
        &self.base.value
    }

    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }

    fn min_length(&self) -> Option<&LengthConstraint> {
        if let FieldKind::Text { config, .. } = &self.kind {
            config.min_length.as_ref()
        } else {
            None
        }
    }

    fn max_length(&self) -> Option<&LengthConstraint> {
        if let FieldKind::Text { config, .. } = &self.kind {
            config.max_length.as_ref()
        } else {
            None
        }
    }

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn is_required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    // Setters
    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        self.base.readonly = Some(BoolChoice {
            choice: readonly,
            message: msg.map(|s| s.to_string()),
        });
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        self.base.disabled = Some(BoolChoice {
            choice: disabled,
            message: msg.map(|s| s.to_string()),
        });
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    // Validation
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        // 1. Vérification requis
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

        // Si vide et non requis, c'est valide
        if val.is_empty() {
            return true;
        }

        // 2. Validation spécifique au type de champ
        if let FieldKind::Text { config, format } = &self.kind {
            // Validation de longueur min
            if let Some(limits) = &config.min_length {
                let count = val.chars().count();
                if let Some(min_val) = limits.min {
                    if count < min_val {
                        let msg = limits
                            .message
                            .clone()
                            .unwrap_or_else(|| format!("Trop court (min {})", min_val));
                        self.set_error(msg);
                        return false;
                    }
                }
            }

            // Validation de longueur max
            if let Some(limits) = &config.max_length {
                let count = val.chars().count();
                if let Some(max_val) = limits.max {
                    if count > max_val {
                        let msg = limits
                            .message
                            .clone()
                            .unwrap_or_else(|| format!("Trop long (max {})", max_val));
                        self.set_error(msg);
                        return false;
                    }
                }
            }

            // Validation du format spécial
            match format {
                SpecialFormat::Email if !val.validate_email() => {
                    self.set_error("Format d'adresse email invalide".into());
                    return false;
                }
                SpecialFormat::Email => {
                    // Normaliser l'email en minuscules après validation
                    self.base.value = val.to_lowercase();
                }
                SpecialFormat::Url if !val.validate_url() => {
                    self.set_error("Veuillez entrer une URL valide".into());
                    return false;
                }
                _ => {}
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();

        // Ne pas afficher la valeur des mots de passe
        if let FieldKind::Text {
            format: SpecialFormat::Password,
            ..
        } = &self.kind
        {
            let mut safe_base = self.base.clone();
            safe_base.value = "".to_string();
            context.insert("field", &safe_base);
        } else {
            context.insert("field", &self.base);
        }

        context.insert("input_type", &self.base.type_field);

        // Ajouter les contraintes de longueur au contexte
        if let FieldKind::Text { config, .. } = &self.kind {
            if let Some(l) = &config.min_length {
                if let Some(min) = l.min {
                    context.insert("min_length", &min);
                }
            }
            if let Some(l) = &config.max_length {
                if let Some(max) = l.max {
                    context.insert("max_length", &max);
                }
            }
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    // Sérialisation JSON pour Tera
    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_readonly(&self) -> Value {
        json!(self.base.readonly)
    }

    fn to_json_disabled(&self) -> Value {
        json!(self.base.disabled)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }

    fn to_json_value(&self) -> Value {
        // Conversion en type JSON approprié selon le type de champ
        match &self.kind {
            FieldKind::Integer(_) => self
                .base
                .value
                .parse::<i64>()
                .map(|v| json!(v))
                .unwrap_or(json!(null)),
            FieldKind::Float(_) => self
                .base
                .value
                .parse::<f64>()
                .map(|v| json!(v))
                .unwrap_or(json!(null)),
            FieldKind::Boolean => {
                json!(self.base.value == "true" || self.base.value == "1")
            }
            FieldKind::Text { .. } => json!(self.base.value),
        }
    }
}
