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
use country::Country;
use postal_code::PostalCode;

// --- FONCTION UTILITAIRE ---
pub fn to_options(opts: Vec<(&str, &str)>) -> Vec<SelectOption> {
    opts.into_iter()
        .map(|(v, l): (&str, &str)| SelectOption {
            // Type du tuple explicite
            value: v.to_string(),
            label: l.to_string(),
        })
        .collect::<Vec<SelectOption>>() // Type de collection explicite
}

// Struct utilitaire
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

pub trait RuniqueField {
    type Output;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;
    fn template_name(&self) -> &str;

    // Indique si on doit trimmer la valeur
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
    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "type": "ip" })
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
    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "type": "domaine" })
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
            "options": self.options.iter().map(|o: &SelectOption| {
                serde_json::json!({
                    "value": o.value,
                    "label": o.label
                })
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

// --- TEXTE AVANCÉ ---

pub struct PhoneField;

impl RuniqueField for PhoneField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Nettoie les espaces, tirets, parenthèses
        let cleaned: String = raw_value
            .chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect();

        // Validation basique : entre 8 et 15 chiffres
        if cleaned.len() < 8 || cleaned.len() > 15 {
            return Err("Numéro de téléphone invalide.".to_string());
        }

        Ok(cleaned)
    }

    fn template_name(&self) -> &str {
        "tel"
    }
}

pub struct ColorField;

impl RuniqueField for ColorField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();

        // Validation format hexadécimal #RRGGBB
        let re = fancy_regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();

        if !re.is_match(val).unwrap_or(false) {
            return Err("Format de couleur invalide (attendu: #RRGGBB).".to_string());
        }

        Ok(val.to_lowercase())
    }

    fn template_name(&self) -> &str {
        "color"
    }
}

pub struct UUIDField;

impl RuniqueField for UUIDField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim().to_lowercase();

        // Validation format UUID
        let re = fancy_regex::Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        )
        .unwrap();

        if !re.is_match(&val).unwrap_or(false) {
            return Err("Format UUID invalide.".to_string());
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "text"
    }
}

// --- NUMÉRIQUE AVANCÉ ---

pub struct DecimalField {
    pub max_digits: usize,
    pub decimal_places: usize,
}

impl DecimalField {
    pub fn new(max_digits: usize, decimal_places: usize) -> Self {
        Self {
            max_digits,
            decimal_places,
        }
    }
}

impl RuniqueField for DecimalField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.replace(',', ".");

        // Valide que c'est un nombre
        let parsed: f64 = val
            .parse()
            .map_err(|_| "Veuillez entrer un nombre décimal valide.".to_string())?;

        // Vérifie le nombre de décimales
        let parts: Vec<&str> = val.split('.').collect();
        if parts.len() > 1 && parts[1].len() > self.decimal_places {
            return Err(format!(
                "Maximum {} décimales autorisées.",
                self.decimal_places
            ));
        }

        // Vérifie le nombre total de chiffres
        let total_digits = val.replace(['.', '-'], "").len();
        if total_digits > self.max_digits {
            return Err(format!("Maximum {} chiffres autorisés.", self.max_digits));
        }

        Ok(format!("{:.prec$}", parsed, prec = self.decimal_places))
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "step": format!("0.{:0>width$}1", "", width = self.decimal_places - 1)
        })
    }
}

pub struct RangeField {
    pub min: i64,
    pub max: i64,
    pub step: i64,
}

impl RangeField {
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max, step: 1 }
    }

    pub fn with_step(min: i64, max: i64, step: i64) -> Self {
        Self { min, max, step }
    }
}

impl RuniqueField for RangeField {
    type Output = i64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: i64 = raw_value
            .parse()
            .map_err(|_| "Valeur numérique invalide.".to_string())?;

        if val < self.min || val > self.max {
            return Err(format!(
                "La valeur doit être comprise entre {} et {}.",
                self.min, self.max
            ));
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "range"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "min": self.min,
            "max": self.max,
            "step": self.step
        })
    }
}

pub struct PositiveIntegerField;

impl RuniqueField for PositiveIntegerField {
    type Output = u64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: u64 = raw_value
            .parse()
            .map_err(|_| "Veuillez entrer un nombre entier positif.".to_string())?;

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({ "min": 0 })
    }
}

pub struct PercentageField;

impl RuniqueField for PercentageField {
    type Output = f64;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val: f64 = raw_value
            .replace(',', ".")
            .parse()
            .map_err(|_| "Veuillez entrer un nombre.".to_string())?;

        if !(0.0..=100.0).contains(&val) {
            return Err("Le pourcentage doit être entre 0 et 100.".to_string());
        }

        Ok(val)
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "min": 0,
            "max": 100,
            "step": 0.01
        })
    }
}

pub struct CurrencyField {
    pub currency: String,
    pub decimal_places: usize,
}

impl CurrencyField {
    pub fn new(currency: &str) -> Self {
        Self {
            currency: currency.to_string(),
            decimal_places: 2, // Par défaut 2 décimales pour la monnaie
        }
    }

    pub fn with_decimal_places(currency: &str, decimal_places: usize) -> Self {
        Self {
            currency: currency.to_string(),
            decimal_places,
        }
    }
}

impl RuniqueField for CurrencyField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let cleaned = raw_value
            .replace(&self.currency, "")
            .replace(' ', "")
            .replace(',', ".");

        let val: f64 = cleaned
            .parse()
            .map_err(|_| "Montant invalide.".to_string())?;

        if val < 0.0 {
            return Err("Le montant ne peut pas être négatif.".to_string());
        }

        // Formatte avec le nombre exact de décimales
        Ok(format!("{:.prec$}", val, prec = self.decimal_places))
    }

    fn template_name(&self) -> &str {
        "number"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "step": format!("0.{:0>width$}1", "", width = self.decimal_places.saturating_sub(1)),
            "min": 0
        })
    }
}

// --- TEMPOREL AVANCÉ ---

pub struct TimeField;

impl RuniqueField for TimeField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Valide le format HH:MM ou HH:MM:SS
        let re =
            fancy_regex::Regex::new(r"^([0-1][0-9]|2[0-3]):[0-5][0-9](:[0-5][0-9])?$").unwrap();

        if !re.is_match(raw_value).unwrap_or(false) {
            return Err("Format d'heure invalide (HH:MM).".to_string());
        }

        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "time"
    }
}

pub struct DurationField;

impl RuniqueField for DurationField {
    type Output = u64; // Durée en secondes

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.to_lowercase();
        let mut total_seconds = 0u64;

        // Parse formats comme "2h30m", "90m", "1h", "3600s"
        let re = fancy_regex::Regex::new(r"(\d+)([hms])").unwrap();

        for cap in re.captures_iter(&val) {
            if let Ok(capture) = cap {
                let num: u64 = capture[1].parse().unwrap_or(0);
                let unit = &capture[2];

                total_seconds += match unit {
                    "h" => num * 3600,
                    "m" => num * 60,
                    "s" => num,
                    _ => 0,
                };
            }
        }
        Ok(total_seconds)
    }

    fn template_name(&self) -> &str {
        "time"
    }
}

// --- FICHIERS AVANCÉS ---

pub struct ImageField {
    pub max_size_mb: f64,
}

impl ImageField {
    pub fn new() -> Self {
        Self { max_size_mb: 5.0 }
    }

    pub fn with_max_size(max_size_mb: f64) -> Self {
        Self { max_size_mb }
    }
}

impl Default for ImageField {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueField for ImageField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Validation basique du nom de fichier
        let lower = raw_value.to_lowercase();
        let valid_extensions = [
            "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", "tiff", "tif", "avif",
            "heic", "heif", "jfif", "pjpeg", "pjp", "apng", "png",
        ];

        if !valid_extensions.iter().any(|ext| lower.ends_with(ext)) {
            return Err("Format d'image non supporté.".to_string());
        }

        Ok(raw_value.to_string())
    }

    fn template_name(&self) -> &str {
        "file"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "accept": "image/*"
        })
    }
}

pub struct MultipleFileField {
    pub accept: String,
    pub max_files: usize,
}

impl MultipleFileField {
    pub fn new(accept: &str) -> Self {
        Self {
            accept: accept.to_string(),
            max_files: 10,
        }
    }

    pub fn with_max_files(accept: &str, max_files: usize) -> Self {
        Self {
            accept: accept.to_string(),
            max_files,
        }
    }
}

impl RuniqueField for MultipleFileField {
    type Output = Vec<String>;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let files: Vec<String> = raw_value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if files.len() > self.max_files {
            return Err(format!("Maximum {} fichiers autorisés.", self.max_files));
        }
        if files.is_empty() {
            return Err("Aucun fichier sélectionné.".to_string());
        }
        Ok(files)
    }

    fn template_name(&self) -> &str {
        "file-multiple"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "accept": self.accept,
            "max_files": self.max_files
        })
    }
}

// --- RELATIONNEL/CHOIX ---

pub struct MultipleChoiceField {
    pub options: Vec<SelectOption>,
}

impl RuniqueField for MultipleChoiceField {
    type Output = Vec<String>;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let selected: Vec<String> = raw_value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Vérifie que toutes les valeurs sont valides
        for val in &selected {
            if !self.options.iter().any(|o| &o.value == val) {
                return Err(format!("Option invalide: {}", val));
            }
        }

        if selected.is_empty() {
            return Err("Veuillez sélectionner au moins une option.".to_string());
        }

        Ok(selected)
    }

    fn template_name(&self) -> &str {
        "select-multiple"
    }

    fn get_context(&self) -> serde_json::Value {
        serde_json::json!({
            "options": self.options.iter().map(|o: &SelectOption| { // Ajout du type explicite ici
                serde_json::json!({
                    "value": o.value,
                    "label": o.label
                })
            }).collect::<Vec<serde_json::Value>>() // Préciser le type de collection aide aussi
        })
    }
}

pub struct RadioSelectField {
    pub options: Vec<SelectOption>,
}

impl RuniqueField for RadioSelectField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if self.options.iter().any(|o| o.value == raw_value) {
            Ok(raw_value.to_string())
        } else {
            Err("Option invalide".to_string())
        }
    }

    fn template_name(&self) -> &str {
        "radio"
    }

    fn get_context(&self) -> serde_json::Value {
        let opts: Vec<serde_json::Value> = self
            .options
            .iter()
            .map(|o: &SelectOption| serde_json::json!({ "value": o.value, "label": o.label }))
            .collect();
        serde_json::json!({ "options": opts })
    }
}

pub struct PostalCodeField {
    pub country: Country,
}

impl PostalCodeField {
    pub fn new(country: Country) -> Self {
        Self { country }
    }
}

impl RuniqueField for PostalCodeField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();

        match PostalCode::new(self.country, val) {
            Ok(postal_code) => Ok(postal_code.to_string()),
            Err(_) => Err("Code postal invalide pour le pays sélectionné.".to_string()),
        }
    }

    fn template_name(&self) -> &str {
        "text"
    }
}
