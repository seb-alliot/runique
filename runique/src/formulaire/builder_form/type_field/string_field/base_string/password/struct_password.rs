use crate::formulaire::builder_form::base_struct::FieldConfig;
use crate::formulaire::builder_form::option_field::{BoolChoice, LengthConstraint};
use crate::formulaire::builder_form::trait_form::FormField;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Context;

#[derive(Clone)]
pub struct PasswordField {
    pub base: FieldConfig,
    pub min_length: Option<LengthConstraint>,
}

impl PasswordField {
    pub fn new(name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                name: name.to_string(),
                label: label.to_string(),
                value: "".to_string(),
                placeholder: "".to_string(),
                is_required: BoolChoice {
                    choice: false,
                    message: None,
                },
                error: None,
                readonly: None,
                disabled: None,
                type_field: "password".to_string(),
                html_attributes: HashMap::new(),
                template_name: "base_string".to_string(),
                extra_context: HashMap::new(),
            },
            min_length: Some(LengthConstraint {
                limit: 8,
                message: None,
            }),
        }
    }

    // Builder cohérent avec le reste du framework
    pub fn required(mut self, msg: &str) -> Self {
        self.base.is_required = BoolChoice {
            choice: true,
            message: if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            },
        };
        self
    }

    pub fn min_length(mut self, limit: usize, msg: &str) -> Self {
        self.min_length = Some(LengthConstraint {
            limit,
            message: if msg.is_empty() {
                None
            } else {
                Some(msg.to_string())
            },
        });
        self
    }

    // Logique Argon2
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
    fn name(&self) -> &str {
        &self.base.name
    }
    fn field_type(&self) -> &str {
        &self.base.type_field
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
    fn is_required(&self) -> bool {
        self.base.is_required.choice
    }
    fn get_error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }
    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }
    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();
        let count = val.chars().count();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Le mot de passe est obligatoire".to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            if let Some(min) = &self.min_length {
                if count < min.limit {
                    let msg = min.message.clone().unwrap_or_else(|| {
                        format!(
                            "Le mot de passe doit faire au moins {} caractères",
                            min.limit
                        )
                    });
                    self.set_error(msg);
                    return false;
                }
            }
        }

        self.set_error("".to_string());
        true
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }
    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }
    fn set_placeholder(&mut self, placeholder: &str) {
        self.base.placeholder = placeholder.to_string();
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

    fn render(&self, tera: &Arc<tera::Tera>) -> Result<String, String> {
        let mut context = Context::new();

        // Pour la sécurité, on peut cloner la base et vider la value avant le rendu
        let mut safe_base = self.base.clone();
        safe_base.value = "".to_string();

        context.insert("field", &safe_base);
        context.insert("input_type", "password");

        tera.render(&self.base.template_name, &context)
            .map_err(|e| format!("Erreur rendu Password: {}", e))
    }

    // Méthodes de sérialisation pour le filtre form
    fn get_is_required_config(&self) -> serde_json::Value {
        serde_json::to_value(&self.base.is_required)
            .unwrap_or(serde_json::json!({"choice": false, "message": null}))
    }

    fn get_readonly_config(&self) -> serde_json::Value {
        if let Some(ref readonly) = self.base.readonly {
            serde_json::to_value(readonly)
                .unwrap_or(serde_json::json!({"choice": false, "message": null}))
        } else {
            serde_json::json!({"choice": false, "message": null})
        }
    }

    fn get_disabled_config(&self) -> serde_json::Value {
        if let Some(ref disabled) = self.base.disabled {
            serde_json::to_value(disabled)
                .unwrap_or(serde_json::json!({"choice": false, "message": null}))
        } else {
            serde_json::json!({"choice": false, "message": null})
        }
    }

    fn get_html_attributes(&self) -> serde_json::Value {
        serde_json::to_value(&self.base.html_attributes).unwrap_or(serde_json::json!({}))
    }
}
