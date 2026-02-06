use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use serde::Serialize;
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use tera::{Context, Tera};
use uuid::Uuid;

/// ColorField - Sélecteur de couleur HTML5
#[derive(Clone, Serialize, Debug)]
pub struct ColorField {
    pub base: FieldConfig,
}

impl ColorField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "color", "base_color"),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn default_color(mut self, color: &str) -> Self {
        // Valider le format hex
        if color.starts_with('#') && (color.len() == 7 || color.len() == 4) {
            self.base.value = color.to_string();
        }
        self
    }
}

impl CommonFieldConfig for ColorField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for ColorField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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

        if !val.is_empty() {
            // Valider le format hexadécimal #RRGGBB ou #RGB
            if !val.starts_with('#') {
                self.set_error("La couleur doit commencer par #".into());
                return false;
            }

            let hex = &val[1..];
            if hex.len() != 6 && hex.len() != 3 {
                self.set_error("Format de couleur invalide (attendu: #RRGGBB ou #RGB)".into());
                return false;
            }

            if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                self.set_error(
                    "La couleur doit contenir uniquement des caractères hexadécimaux".into(),
                );
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}

/// SlugField - Champ pour slugs URL-friendly
#[derive(Clone, Serialize, Debug)]
pub struct SlugField {
    pub base: FieldConfig,
    pub allow_unicode: bool,
}

impl CommonFieldConfig for SlugField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl SlugField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special"),
            allow_unicode: false,
        }
    }

    pub fn allow_unicode(mut self) -> Self {
        self.allow_unicode = true;
        self
    }
    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.set_label(label);
        self
    }
}

impl FormField for SlugField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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

        if !val.is_empty() {
            // Validation du slug
            if self.allow_unicode {
                // Slug unicode : lettres, chiffres, tirets, underscores
                let valid = val
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
                if !valid {
                    self.set_error(
                        "Le slug ne peut contenir que des lettres, chiffres, tirets et underscores"
                            .into(),
                    );
                    return false;
                }
            } else {
                // Slug ASCII uniquement
                let valid = val
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
                if !valid {
                    self.set_error("Le slug ne peut contenir que des caractères ASCII, chiffres, tirets et underscores".into());
                    return false;
                }
            }

            // Ne doit pas commencer ou finir par un tiret
            if val.starts_with('-') || val.ends_with('-') {
                self.set_error("Le slug ne peut pas commencer ou finir par un tiret".into());
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("field_hint", &"Format: lettres-chiffres-tirets");

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}

/// UUIDField - Champ pour identifiants UUID
#[derive(Clone, Serialize, Debug)]
pub struct UUIDField {
    pub base: FieldConfig,
}

impl UUIDField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special"),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }
}

impl CommonFieldConfig for UUIDField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for UUIDField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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

        if !val.is_empty() {
            // Valider le format UUID
            if Uuid::parse_str(val).is_err() {
                self.set_error(
                    "Format UUID invalide (attendu: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)".into(),
                );
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert(
            "field_hint",
            &"Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        );

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}

/// JSONField - Textarea avec validation JSON
#[derive(Clone, Serialize, Debug)]
pub struct JSONField {
    pub base: FieldConfig,
}

impl JSONField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "textarea", "base_special"),
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.base
            .extra_context
            .insert("rows".to_string(), json!(rows));
        self
    }
}

impl CommonFieldConfig for JSONField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for JSONField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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

        if !val.is_empty() {
            // Valider le JSON
            if serde_json::from_str::<serde_json::Value>(val).is_err() {
                self.set_error("JSON invalide".into());
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("field_hint", &"Format JSON valide requis");
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());
        // Nombre de lignes
        let rows = self
            .base
            .extra_context
            .get("rows")
            .and_then(|r| r.as_u64().map(|v| v as usize))
            .unwrap_or(10);
        context.insert("rows", &rows);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}

/// IPAddressField - Validation d'adresse IP (v4 ou v6)
#[derive(Clone, Serialize, Debug)]
pub struct IPAddressField {
    pub base: FieldConfig,
    pub ipv6_only: bool,
    pub ipv4_only: bool,
}

impl IPAddressField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special"),
            ipv6_only: false,
            ipv4_only: false,
        }
    }

    pub fn ipv4_only(mut self) -> Self {
        self.ipv4_only = true;
        self.ipv6_only = false;
        self
    }

    pub fn ipv6_only(mut self) -> Self {
        self.ipv6_only = true;
        self.ipv4_only = false;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }
}

impl CommonFieldConfig for IPAddressField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for IPAddressField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

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

        if !val.is_empty() {
            // Parser l'adresse IP
            match val.parse::<IpAddr>() {
                Ok(ip) => {
                    if self.ipv4_only && ip.is_ipv6() {
                        self.set_error("Seules les adresses IPv4 sont acceptées".into());
                        return false;
                    }
                    if self.ipv6_only && ip.is_ipv4() {
                        self.set_error("Seules les adresses IPv6 sont acceptées".into());
                        return false;
                    }
                }
                Err(_) => {
                    self.set_error("Adresse IP invalide".into());
                    return false;
                }
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        let hint = if self.ipv4_only {
            "Format IPv4: 192.168.1.1"
        } else if self.ipv6_only {
            "Format IPv6: 2001:0db8:85a3::8a2e:0370:7334"
        } else {
            "Format IPv4 ou IPv6"
        };
        context.insert("field_hint", &hint);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }
}
