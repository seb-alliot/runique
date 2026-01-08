use crate::formulaire::sanetizer;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Algorithm, Argon2, Params, Version,
};
use fancy_regex::Regex;
use serde_json::Value;
use std::net::IpAddr;
use tera::Context;
use tera::Tera;

use chrono::{NaiveDate, NaiveDateTime};

pub trait RuniqueField {
    type Output;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;
    fn template_name(&self) -> &str;

    // Indique si on doit trimmer la valeur (utile pour ton Forms::field)
    fn strip(&self) -> bool {
        true
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// La méthode qui rend le template Tera
    fn render(
        &self,
        tera: &Tera,
        name: &str,
        label: &str,
        value: &Value,
        error: Option<&String>,
    ) -> String {
        let mut ctx = Context::new();
        ctx.insert("name", name);
        ctx.insert("label", label);
        ctx.insert("id", &format!("id_{}", name));
        ctx.insert("value", value);

        if let Some(err) = error {
            ctx.insert("error", err);
        }

        // On injecte les données spécifiques (comme 'accept' ou 'options')
        if let Value::Object(map) = self.get_context() {
            for (k, v) in map {
                ctx.insert(k, &v);
            }
        }

        tera.render(self.template_name(), &ctx)
            .unwrap_or_else(|e| format!("Erreur Tera ({}): {}", self.template_name(), e))
    }
}

// --- TEXTE ET SECURITE ---

pub struct CharField {
    pub allow_blank: bool,
}

pub struct TextField {
    pub allow_blank: bool,
}

impl Default for CharField {
    fn default() -> Self {
        Self::new()
    }
}

impl CharField {
    pub fn new() -> Self {
        Self { allow_blank: false }
    }

    pub fn allow_blank() -> Self {
        Self { allow_blank: true }
    }
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl TextField {
    pub fn new() -> Self {
        Self { allow_blank: false }
    }

    pub fn allow_blank() -> Self {
        Self { allow_blank: true }
    }
}

impl RuniqueField for CharField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas être vide".to_string());
        }

        Ok(sanetizer::auto_sanitize(raw_value))
    }
    fn template_name(&self) -> &str {
        "text"
    }
}

impl RuniqueField for TextField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas être vide".to_string());
        }

        Ok(sanetizer::auto_sanitize(raw_value))
    }
    fn template_name(&self) -> &str {
        "textarea"
    }
}

pub struct PasswordField;

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

pub struct EmailField;

impl RuniqueField for EmailField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim().to_lowercase();

        let re: Regex =
            fancy_regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if !re.is_match(&val).unwrap_or(false) {
            return Err("Format d'email invalide.".to_string());
        }

        Ok(val)
    }
    fn template_name(&self) -> &str {
        "email"
    }
}

// --- NUMERIQUE ET LOGIQUE ---

pub struct IntegerField;

impl RuniqueField for IntegerField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .parse::<i64>()
            .map_err(|_| "Veuillez entrer un nombre entier.".to_string())
    }
    fn template_name(&self) -> &str {
        "number"
    }
}

pub struct FloatField;

impl RuniqueField for FloatField {
    type Output = f64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .replace(',', ".")
            .parse::<f64>()
            .map_err(|_| "Veuillez entrer un nombre décimal.".to_string())
    }
    fn template_name(&self) -> &str {
        "number"
    }
}

pub struct BooleanField;

impl RuniqueField for BooleanField {
    type Output = bool;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        match raw_value.to_lowercase().as_str() {
            "on" | "true" | "1" | "yes" | "checked" => Ok(true),

            _ => Ok(false),
        }
    }
    fn template_name(&self) -> &str {
        "checkbox"
    }
}

// --- TEMPOREL ---

pub struct DateField;

impl RuniqueField for DateField {
    type Output = NaiveDate;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        NaiveDate::parse_from_str(raw_value, "%Y-%m-%d")
            .map_err(|_| "Format de date invalide (AAAA-MM-JJ).".to_string())
    }
    fn template_name(&self) -> &str {
        "date"
    }
}

pub struct DateTimeField;

impl RuniqueField for DateTimeField {
    type Output = NaiveDateTime;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est déjÃ  trimmed

        let val = raw_value.replace('T', " ");

        NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M"))
            .map_err(|_| "Format date/heure invalide.".to_string())
    }
    fn template_name(&self) -> &str {
        "datetime-local"
    }
}

// --- RÉSEAU ET DONNÉES ---

pub struct IPAddressField;

impl RuniqueField for IPAddressField {
    type Output = IpAddr;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est déjÃ  trimmed

        raw_value
            .parse::<IpAddr>()
            .map_err(|_| "Adresse IP invalide.".to_string())
    }
    fn template_name(&self) -> &str {
        "ipaddress"
    }
}

pub struct JSONField;

impl RuniqueField for JSONField {
    type Output = Value;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        serde_json::from_str(raw_value).map_err(|_| "Contenu JSON malformé.".to_string())
    }
    fn template_name(&self) -> &str {
        "json"
    }
}

pub struct URLField;

impl RuniqueField for URLField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.starts_with("http") {
            Ok(raw_value.to_string())
        } else {
            Err("L'URL doit commencer par http:// ou https://".to_string())
        }
    }
    fn template_name(&self) -> &str {
        "url"
    }
}

pub struct SlugField;

impl RuniqueField for SlugField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let slug = raw_value
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        if slug.is_empty() {
            return Err("Le titre ne peut pas être vide.".to_string());
        }

        Ok(slug)
    }
    fn template_name(&self) -> &str {
        "slug"
    }
}

pub struct FileField {
    pub accept: String,
}

impl RuniqueField for FileField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "file"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "accept": self.accept })
    }
}

pub struct SelectOption {
    pub value: String,
    pub label: String,
}

pub struct SelectField {
    pub options: Vec<SelectOption>,
}

impl RuniqueField for SelectField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if self.options.iter().any(|o| o.value == raw_value) {
            Ok(raw_value.to_string())
        } else {
            Err("Option invalide".to_string())
        }
    }

    fn template_name(&self) -> &str {
        "select"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "options": self.options.iter().map(|o| {
                serde_json::json!({ "value": o.value, "label": o.label })
            }).collect::<Vec<_>>()
        })
    }
}

pub struct HiddenField;

impl RuniqueField for HiddenField {
    type Output = String;
    fn process(&self, v: &str) -> Result<Self::Output, String> {
        Ok(v.to_string())
    }

    fn template_name(&self) -> &str {
        "hidden"
    }
}
