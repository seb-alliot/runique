use crate::formulaire::sanetizer;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};


use serde_json::Value;
use std::net::IpAddr;
use chrono::{NaiveDate, NaiveDateTime};

/// Le Trait maître : définit comment une donnée brute devient une donnée Rusti
pub trait RustiField {
    type Output;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String>;
}

// --- TEXTE ET SÉCURITÉ ---

pub struct CharField;

impl RustiField for CharField {
    type Output = String;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let clean = sanetizer::auto_sanitize(raw_value.trim());
        Ok(clean)
    }
}

pub struct TextField;

impl RustiField for TextField {
    type Output = String;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Ici, on imagine que sanetizer supporte une option pour garder les \n
        Ok(sanetizer::auto_sanitize(raw_value))
    }
}

pub struct PasswordField;

impl RustiField for PasswordField {
    type Output = String;

    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        if raw_value.len() < 8 {
            return Err("Le mot de passe doit contenir au moins 8 caractères.".to_string());
        }

        // 1. Générer un sel aléatoire (Salt) unique pour ce mot de passe
        let salt = SaltString::generate(&mut OsRng);

        // 2. Initialiser Argon2 avec les paramètres par défaut (Argon2id)
        let argon2 = Argon2::default();

        // 3. Hacher le mot de passe
        let password_hash = argon2.hash_password(raw_value.as_bytes(), &salt)
            .map_err(|_| "Erreur lors du hachage du mot de passe".to_string())?
            .to_string(); // Retourne le hash au format PHC (ex: $argon2id$v=19$m=4096...)

        Ok(password_hash)
    }
}

pub struct EmailField;

impl RustiField for EmailField {
    type Output = String;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let email = raw_value.trim().to_lowercase();
        if email.contains('@') && email.contains('.') && email.len() > 5 {
            Ok(email)
        } else {
            Err("Format d'email invalide.".to_string())
        }
    }
}

// --- NUMÉRIQUE ET LOGIQUE ---

pub struct IntegerField;

impl RustiField for IntegerField {
    type Output = i64;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value.trim().parse::<i64>()
            .map_err(|_| "Ce champ doit être un nombre entier.".to_string())
    }
}

pub struct FloatField;

impl RustiField for FloatField {
    type Output = f64;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value.trim().replace(',', ".").parse::<f64>()
            .map_err(|_| "Nombre décimal invalide.".to_string())
    }
}

pub struct BooleanField;

impl RustiField for BooleanField {
    type Output = bool;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        match raw_value.to_lowercase().trim() {
            "on" | "true" | "1" | "yes" | "checked" => Ok(true),
            _ => Ok(false),
        }
    }
}

// --- TEMPOREL ---

pub struct DateField;

impl RustiField for DateField {
    type Output = NaiveDate;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        NaiveDate::parse_from_str(raw_value.trim(), "%d-%m-%Y")
            .map_err(|_| "Format de date invalide (JJ-MM-AAAA).".to_string())
    }
}

pub struct DateTimeField;

impl RustiField for DateTimeField {
    type Output = NaiveDateTime;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        // Supporte souvent le format T des navigateurs (2025-12-22T14:00)
        let val = raw_value.trim().replace('T', " ");
        NaiveDateTime::parse_from_str(&val, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| "Format date/heure invalide.".to_string())
    }
}

// --- RÉSEAU ET DONNÉES ---

pub struct IPAddressField;
impl RustiField for IPAddressField {
    type Output = IpAddr;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        raw_value.trim().parse::<IpAddr>()
            .map_err(|_| "Adresse IP invalide.".to_string())
    }
}

pub struct JSONField;
impl RustiField for JSONField {
    type Output = Value;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        serde_json::from_str(raw_value)
            .map_err(|_| "Contenu JSON malformé.".to_string())
    }
}

pub struct URLField;
impl RustiField for URLField {
    type Output = String;
    fn process(&self, raw_value: &str) -> Result<Self::Output, String> {
        let val = raw_value.trim();
        if val.starts_with("http") {
            Ok(val.to_string())
        } else {
            Err("L'URL doit commencer par http:// ou https://".to_string())
        }
    }
}