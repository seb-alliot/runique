use crate::formulaire::sanetizer;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use serde_json::Value;

use std::net::IpAddr;

use chrono::{NaiveDate, NaiveDateTime};

pub trait RustiField {
    type Output;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;

    fn strip(&self) -> bool {
        true
    }

    fn render_html(&self, field_name: &str, label: &str) -> String;
}

// --- TEXTE ET SÃ‰CURITÃ‰ ---

pub struct CharField {
    pub allow_blank: bool,
}

pub struct TextField {
    pub allow_blank: bool,
}

impl CharField {
    pub fn new() -> Self {
        Self { allow_blank: false }
    }

    pub fn allow_blank() -> Self {
        Self { allow_blank: true }
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

impl RustiField for CharField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas Ãªtre vide".to_string());
        }

        Ok(sanetizer::auto_sanitize(raw_value))
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        let required = if self.allow_blank { "" } else { "required" };

        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="text"

                name="{}"

                id="id_{}"

                class="form-control"

                {}>

            </div>"#,
            field_name, label, field_name, field_name, required
        )
    }
}

impl RustiField for TextField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if !self.allow_blank && raw_value.is_empty() {
            return Err("Ce champ ne peut pas Ãªtre vide".to_string());
        }

        Ok(sanetizer::auto_sanitize(raw_value))
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        let required = if self.allow_blank { "" } else { "required" };

        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="text"

                name="{}"

                id="id_{}"

                class="form-control"

                {}>

            </div>"#,
            field_name, label, field_name, field_name, required
        )
    }
}

pub struct PasswordField;

impl RustiField for PasswordField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.len() < 8 {
            return Err("Le mot de passe doit contenir au moins 8 caractÃ¨res.".to_string());
        }

        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(raw_value.as_bytes(), &salt)
            .map_err(|_| "Erreur lors du hachage du mot de passe".to_string())?
            .to_string();

        Ok(password_hash)
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="password"

                name="{}"

                id="id_{}"

                class="form-control"

                >

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

pub struct EmailField;

impl RustiField for EmailField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.contains('@') && raw_value.contains('.') && raw_value.len() > 5 {
            Ok(raw_value.to_lowercase()) // Lowercase pour email
        } else {
            Err("Format d'email invalide.".to_string())
        }
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="email"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

// --- NUMÃ‰RIQUE ET LOGIQUE ---

pub struct IntegerField;

impl RustiField for IntegerField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .parse::<i64>()
            .map_err(|_| "EntrÃ© un nombre entier.".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="number"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name,
        )
    }
}

pub struct FloatField;

impl RustiField for FloatField {
    type Output = f64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value
            .replace(',', ".")
            .parse::<f64>()
            .map_err(|_| "EntrÃ© un nombre dÃ©cimal.".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="number" step="any"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name,
        )
    }
}

pub struct BooleanField;

impl RustiField for BooleanField {
    type Output = bool;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        match raw_value.to_lowercase().as_str() {
            "on" | "true" | "1" | "yes" | "checked" => Ok(true),

            _ => Ok(false),
        }
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group form-check">

                <input type="checkbox"

                name="{}"

                id="id_{}"

                class="form-check-input">

                <label for="id_{}" class="form-check-label">{}</label>

            </div>"#,
            field_name, field_name, field_name, label,
        )
    }
}

// --- TEMPOREL ---

pub struct DateField;

impl RustiField for DateField {
    type Output = NaiveDate;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est dÃ©jÃ  trimmed

        NaiveDate::parse_from_str(raw_value, "%Y-%m-%d")
            .map_err(|_| "Format de date invalide (AAAA-MM-JJ).".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group

                <label for="id_{}">{}</label>

                <input type="date"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

pub struct DateTimeField;

impl RustiField for DateTimeField {
    type Output = NaiveDateTime;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est dÃ©jÃ  trimmed

        let val = raw_value.replace('T', " ");

        NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| "Format date/heure invalide.".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="datetime-local"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

// --- RÃ‰SEAU ET DONNÃ‰ES ---

pub struct IPAddressField;

impl RustiField for IPAddressField {
    type Output = IpAddr;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // raw_value est dÃ©jÃ  trimmed

        raw_value
            .parse::<IpAddr>()
            .map_err(|_| "Adresse IP invalide.".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="text"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

pub struct JSONField;

impl RustiField for JSONField {
    type Output = Value;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        serde_json::from_str(raw_value).map_err(|_| "Contenu JSON malformÃ©.".to_string())
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <textarea

                name="{}"

                id="id_{}"

                class="form-control"

                rows="5"></textarea>

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

pub struct URLField;

impl RustiField for URLField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.starts_with("http") {
            Ok(raw_value.to_string())
        } else {
            Err("L'URL doit commencer par http:// ou https://".to_string())
        }
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="url"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}

pub struct SlugField;

impl RustiField for SlugField {
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
            return Err("Le titre ne peut pas Ãªtre vide.".to_string());
        }

        Ok(slug)
    }

    fn render_html(&self, field_name: &str, label: &str) -> String {
        format!(
            r#"

            <div class="form-group">

                <label for="id_{}">{}</label>

                <input type="text"

                name="{}"

                id="id_{}"

                class="form-control">

            </div>"#,
            field_name, label, field_name, field_name
        )
    }
}
