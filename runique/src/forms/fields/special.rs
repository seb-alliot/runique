//! Special fields: `ColorField`, `UUIDField`, `IPAddressField`, `JSONField`, `SlugField`.
use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::json;
use std::{net::IpAddr, sync::Arc};
use tera::Tera;
use uuid::Uuid;

/// ColorField - HTML5 color selector
#[derive(Clone, Serialize, Debug)]
pub struct ColorField {
    pub base: FieldConfig,
}

impl ColorField {
    /// Creates a new color field rendered as an HTML5 `<input type="color">`.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "color", "base_color.html"),
        }
    }

    /// Sets the field's display label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required; empty input fails validation.
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the default color value. Only accepted if it matches the `#RGB` or
    /// `#RRGGBB` hex format; otherwise the value passed in is silently ignored.
    pub fn default_color(mut self, color: &str) -> Self {
        // Validate the hex format
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

#[async_trait]
impl FormField for ColorField {
    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Validate the hexadecimal format #RRGGBB or #RGB
            if !val.starts_with('#') {
                self.set_error(t("forms.color_no_hash").to_string());
                return false;
            }

            let hex = &val[1..];
            if hex.len() != 6 && hex.len() != 3 {
                self.set_error(t("forms.color_invalid").to_string());
                return false;
            }

            if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                self.set_error(t("forms.color_bad_hex").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let context = self.base_context();

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

/// SlugField - Field for URL-friendly slugs
#[derive(Clone, Serialize, Debug)]
pub struct SlugField {
    pub base: FieldConfig,
    /// When `true`, allows unicode letters and digits in the slug in addition
    /// to `-` and `_`; when `false` (the default), only ASCII alphanumerics are allowed.
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
    /// Creates a new slug field with ASCII-only validation by default.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special.html"),
            allow_unicode: false,
        }
    }

    /// Enables unicode letters and digits in slug validation, instead of
    /// restricting to ASCII alphanumerics.
    pub fn allow_unicode(mut self) -> Self {
        self.allow_unicode = true;
        self
    }
    /// Sets the placeholder text shown in the empty input.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    /// Sets the field's display label.
    pub fn label(mut self, label: &str) -> Self {
        self.set_label(label);
        self
    }
}

#[async_trait]
impl FormField for SlugField {
    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Slug validation
            if self.allow_unicode {
                // Unicode slug: letters, numbers, dashes, underscores
                let valid = val
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
                if !valid {
                    self.set_error(t("forms.slug_unicode_invalid").to_string());
                    return false;
                }
            } else {
                // ASCII slug only
                let valid = val
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
                if !valid {
                    self.set_error(t("forms.slug_invalid").to_string());
                    return false;
                }
            }

            // Must not start or end with a dash
            if val.starts_with('-') || val.ends_with('-') {
                self.set_error(t("forms.slug_no_dash").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("field_hint", &t("forms.hint_slug").to_string());

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

/// UUIDField - Field for UUID identifiers
#[derive(Clone, Serialize, Debug)]
pub struct UUIDField {
    pub base: FieldConfig,
}

impl UUIDField {
    /// Creates a new UUID field rendered as a plain text input.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special.html"),
        }
    }

    /// Sets the field's display label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required; empty input fails validation.
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the placeholder text shown in the empty input.
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

#[async_trait]
impl FormField for UUIDField {
    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Validate the UUID format
            if Uuid::parse_str(val).is_err() {
                self.set_error(t("forms.uuid_invalid").to_string());
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("field_hint", &t("forms.hint_uuid").to_string());

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

/// JSONField - Textarea with JSON validation
#[derive(Clone, Serialize, Debug)]
pub struct JSONField {
    pub base: FieldConfig,
}

impl JSONField {
    /// Creates a new JSON field rendered as a textarea.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "textarea", "base_special.html"),
        }
    }

    /// Sets the field's display label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required; empty input fails validation.
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the placeholder text shown in the empty textarea.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    /// Sets the number of visible rows on the rendered textarea (defaults to 10 if never set).
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

#[async_trait]
impl FormField for JSONField {
    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Validate the JSON
            if let Err(e) = serde_json::from_str::<serde_json::Value>(val) {
                self.set_error(tf("forms.json_invalid", &[&e]));
                return false;
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("field_hint", &t("forms.hint_json").to_string());
        // Number of lines
        let rows = self
            .base
            .extra_context
            .get("rows")
            .and_then(|r| r.as_u64().map(|v| v as usize))
            .unwrap_or(10);
        context.insert("rows", &rows);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}

/// IPAddressField - IP address validation (v4 or v6)
#[derive(Clone, Serialize, Debug)]
pub struct IPAddressField {
    pub base: FieldConfig,
    /// When `true`, restricts validation to IPv6 addresses only. Set via
    /// [`IPAddressField::ipv6_only`].
    pub ipv6_only: bool,
    /// When `true`, restricts validation to IPv4 addresses only. Set via
    /// [`IPAddressField::ipv4_only`].
    pub ipv4_only: bool,
}

impl IPAddressField {
    /// Creates a new IP address field accepting both IPv4 and IPv6 by default.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "text", "base_special.html"),
            ipv6_only: false,
            ipv4_only: false,
        }
    }

    /// Restricts validation to IPv4 addresses only, clearing any prior `ipv6_only` setting.
    pub fn ipv4_only(mut self) -> Self {
        self.ipv4_only = true;
        self.ipv6_only = false;
        self
    }

    /// Restricts validation to IPv6 addresses only, clearing any prior `ipv4_only` setting.
    pub fn ipv6_only(mut self) -> Self {
        self.ipv6_only = true;
        self.ipv4_only = false;
        self
    }

    /// Sets the field's display label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required; empty input fails validation.
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the placeholder text shown in the empty input.
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

#[async_trait]
impl FormField for IPAddressField {
    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if !val.is_empty() {
            // Parse the IP address
            match val.parse::<IpAddr>() {
                Ok(ip) => {
                    if self.ipv4_only && ip.is_ipv6() {
                        self.set_error(t("forms.ipv4_only").to_string());
                        return false;
                    }
                    if self.ipv6_only && ip.is_ipv4() {
                        self.set_error(t("forms.ipv6_only").to_string());
                        return false;
                    }
                }
                Err(_) => {
                    self.set_error(t("forms.ip_invalid").to_string());
                    return false;
                }
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = self.base_context();

        let hint = if self.ipv4_only {
            t("forms.hint_ip_v4").to_string()
        } else if self.ipv6_only {
            t("forms.hint_ip_v6").to_string()
        } else {
            t("forms.hint_ip").to_string()
        };
        context.insert("field_hint", &hint);

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}
