use crate::formulaire::builder_form::base_struct::*;
use crate::formulaire::builder_form::option_field::{BoolChoice, LengthConstraint};
use crate::formulaire::builder_form::trait_form::FormField;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};
use validator::{ValidateEmail, ValidateUrl};

#[derive(Clone, Debug)]
pub enum SpecialFormat {
    None,
    Email,
    Url,
    Password,
    RichText,
}

#[derive(Clone, Debug)]
pub enum FieldKind {
    Text {
        config: TextConfig,
        format: SpecialFormat,
    },
    Integer(IntConfig),
    Float(FloatConfig),
    Boolean,
}

#[derive(Clone)]
pub struct GenericField {
    pub base: FieldConfig,
    pub kind: FieldKind,
}

impl GenericField {
    // Constructeur de base
    fn create(name: &str, label: &str, type_field: &str, kind: FieldKind) -> Self {
        Self {
            base: FieldConfig {
                name: name.to_string(),
                label: label.to_string(),
                value: String::new(),
                placeholder: String::new(),
                is_required: BoolChoice {
                    choice: false,
                    message: None,
                },
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
    // Constructeur de string
    pub fn new_text(name: &str, label: &str) -> Self {
        Self::create(
            name,
            label,
            "text",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::None,
            },
        )
    }

    pub fn new_textarea(name: &str, label: &str) -> Self {
        let mut f = Self::create(
            name,
            label,
            "textarea",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::None,
            },
        );
        f.base.template_name = "base_string".to_string();
        f
    }

    pub fn new_richtext(name: &str, label: &str) -> Self {
        let mut f = Self::create(
            name,
            label,
            "richtext",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::RichText,
            },
        );
        f.base.template_name = "base_string".to_string();
        f
    }

    pub fn new_password(name: &str, label: &str) -> Self {
        Self::create(
            name,
            label,
            "password",
            FieldKind::Text {
                config: TextConfig { max_length: None },
                format: SpecialFormat::Password,
            },
        )
    }

    pub fn new_email(name: &str, label: &str) -> Self {
        let mut f = Self::create(
            name,
            label,
            "email",
            FieldKind::Text {
                config: TextConfig::default(),
                format: SpecialFormat::Email,
            },
        );
        f.base.placeholder = "exemple@domaine.com".to_string();
        f
    }

    pub fn new_url(name: &str, label: &str) -> Self {
        Self::create(
            name,
            label,
            "url",
            FieldKind::Text {
                config: TextConfig {
                    max_length: Some(LengthConstraint {
                        min: None,
                        max: Some(2048),
                        message: None,
                    }),
                },
                format: SpecialFormat::Url,
            },
        )
    }

    pub fn new_int(name: &str, label: &str) -> Self {
        Self::create(
            name,
            label,
            "number",
            FieldKind::Integer(IntConfig::default()),
        )
    }

    // Utilitaires
    //  Logique spécifique pour le mot de passe
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

    pub fn verify(password_plain: &str, password_hash: &str) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password_plain.as_bytes(), &parsed_hash)
            .is_ok()
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    pub fn limites_caractere(mut self, min: Option<usize>, max: Option<usize>, msg: &str) -> Self {
        if let FieldKind::Text { ref mut config, .. } = self.kind {
            config.max_length = Some(LengthConstraint {
                min,
                max,
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

// --- TOUS LES SETTERS ET MÉTHODES DU TRAIT ---
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
    fn field_type(&self) -> &str {
        &self.base.type_field
    }
    fn is_required(&self) -> bool {
        self.base.is_required.choice
    }
    fn get_error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    // Setters (Ceux que tu utilisais dans tes anciens fichiers)
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

        // 1. Requis
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

        // 2. Logique Textuelle
        if let FieldKind::Text { config, format } = &self.kind {
            if let Some(limits) = &config.max_length {
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

            match format {
                SpecialFormat::Email if !val.validate_email() => {
                    self.set_error("Format d'adresse email invalide".into());
                    return false;
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

        if let FieldKind::Text { config, .. } = &self.kind {
            if let Some(l) = &config.max_length {
                if let Some(min) = l.min {
                    context.insert("min_length", &min);
                }
                if let Some(max) = l.max {
                    context.insert("max_length", &max);
                }
            }
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    // Sérialisation JSON pour les filtres Tera
    fn get_is_required_config(&self) -> Value {
        json!(self.base.is_required)
    }
    fn get_readonly_config(&self) -> Value {
        json!(self.base.readonly)
    }
    fn get_disabled_config(&self) -> Value {
        json!(self.base.disabled)
    }
    fn get_html_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}
