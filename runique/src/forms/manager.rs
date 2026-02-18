use crate::forms::base::FormField;
use crate::forms::fields::TextField;
use crate::forms::generic::GenericField;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, trace, warn};

use crate::utils::aliases::{ATera, FieldsMap, JsonMap, OATera, StrMap};
use crate::utils::constante::{
    CONSTRAINT_REGEX, CSRF_TOKEN_KEY, FAILED_REGEX, FOR_KEY_REGEX, KEY_REGEX,
};

// Erreurs possibles lors de la validation du formulaire liée a la bdd
#[derive(Debug, Clone)]
pub enum ValidationError {
    StackOverflow,
    FieldValidation(StrMap),
    GlobalErrors(Vec<String>),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::StackOverflow => {
                write!(
                    f,
                    "Stack overflow détecté : récursion infinie dans la validation"
                )
            }
            ValidationError::FieldValidation(errors) => {
                write!(f, "Erreurs de validation : {:?}", errors)
            }
            ValidationError::GlobalErrors(errors) => {
                write!(f, "Erreurs globales : {}", errors.join(", "))
            }
        }
    }
}

impl std::error::Error for ValidationError {}

thread_local! {
    static VALIDATION_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// Profondeur maximale d'appels récursifs à is_valid()
const MAX_VALIDATION_DEPTH: usize = 20;

#[derive(Clone)]
pub struct Forms {
    pub fields: FieldsMap,
    pub tera: OATera,
    pub global_errors: Vec<String>,
    pub session_csrf_token: Option<String>,
    pub js_files: Vec<String>,
}

impl std::fmt::Debug for Forms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forms")
            .field("fields_count", &self.fields.len())
            .field("has_tera", &self.tera.is_some())
            .field("global_errors", &self.global_errors)
            .finish()
    }
}

impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 7)?;

        state.serialize_field("data", &self.data())?;
        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;
        state.serialize_field("cleaned_data", &self.data())?;
        state.serialize_field("js_files", &self.js_files)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Render error: {}</p>", e),
        };

        state.serialize_field("html", &rendered_html)?;
        let rendered_fields: HashMap<String, String> = self
            .fields
            .iter()
            .filter_map(|(name, field)| {
                let tera_instance = self.tera.as_ref()?;
                field
                    .render(tera_instance)
                    .ok()
                    .map(|html| (name.clone(), html))
            })
            .collect();
        state.serialize_field("rendered_fields", &rendered_fields)?;

        let fields_data: HashMap<String, serde_json::Value> = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, (name, field))| {
                let mut field_map = serde_json::Map::new();
                field_map.insert("name".to_string(), json!(name));
                field_map.insert("label".to_string(), json!(field.label()));
                field_map.insert("field_type".to_string(), json!(field.field_type()));
                field_map.insert("template_name".to_string(), json!(field.template_name()));
                field_map.insert("value".to_string(), json!(field.value()));
                field_map.insert("placeholder".to_string(), json!(field.placeholder()));
                field_map.insert("index".to_string(), json!(index));

                field_map.insert("is_required".to_string(), field.to_json_required());
                field_map.insert("readonly".to_string(), field.to_json_readonly());
                field_map.insert("disabled".to_string(), field.to_json_disabled());
                field_map.insert("html_attributes".to_string(), field.to_json_attributes());
                field_map.insert("meta".to_string(), field.to_json_meta());
                if let Some(err) = field.error() {
                    field_map.insert("error".to_string(), json!(err));
                }
                (name.clone(), Value::Object(field_map))
            })
            .collect();

        state.serialize_field("fields", &fields_data)?;
        state.end()
    }
}

impl Forms {
    pub fn new(csrf_token: &str) -> Self {
        let mut fields: FieldsMap = IndexMap::new();

        // Créer le champ CSRF
        let mut csrf_field = TextField::create_csrf();
        csrf_field.set_value(csrf_token);

        fields.insert(
            CSRF_TOKEN_KEY.to_string(),
            Box::new(csrf_field) as Box<dyn FormField>,
        );

        Self {
            fields,
            tera: None,
            global_errors: Vec::new(),
            session_csrf_token: Some(csrf_token.to_string()),
            js_files: Vec::new(),
        }
    }

    fn render_js(&self, tera: &ATera) -> Result<String, String> {
        if self.js_files.is_empty() {
            return Ok(String::new());
        }

        let template_name = "js_files";

        if !tera.get_template_names().any(|name| name == template_name) {
            return Err(format!("Template manquant: {}", template_name));
        }

        let mut context = tera::Context::new();
        context.insert("js_files", &self.js_files);

        let result = tera
            .render(template_name, &context)
            .map_err(|e| format!("Erreur rendu JS: {}", e))?;

        Ok(result)
    }

    /// La solution au "type annotations needed" :
    /// On force la conversion en GenericField ici même.
    pub fn field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + Into<GenericField> + 'static,
    {
        let generic_instance: GenericField = field_template.clone().into();
        self.fields.insert(
            generic_instance.name().to_string(),
            Box::new(generic_instance),
        );
    }

    // helper pour ajouter un a plusieurs fichiers JS d'un coup
    pub fn add_js(&mut self, files: &[&str]) {
        debug!(files_count = files.len(), "add files JS to form");

        for file in files {
            // Validation
            if let Some(reason) = Self::validate_js_path(file) {
                warn!(file = %file, reason = reason, "Skipping JS file");
                continue;
            }

            // OK
            self.js_files.push(file.to_string());
            trace!(file = %file, "Added JS file to form");
        }
    }

    /// Valide un chemin JS, retourne Some(reason) si invalide
    fn validate_js_path(file: &str) -> Option<&'static str> {
        if !file.ends_with(".js") {
            return Some("File does not have .js extension");
        }

        if file.starts_with('/') || file.starts_with('\\') {
            return Some("Absolute paths are not allowed");
        }

        if file.contains("../") {
            return Some("Path traversal (../) is not allowed");
        }

        None // Valide
    }

    pub fn set_tera(&mut self, tera: ATera) {
        self.tera = Some(tera);
    }

    pub fn fill(&mut self, data: &StrMap) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }
    pub fn finalize(&mut self) -> Result<(), String> {
        for (name, field) in self.fields.iter_mut() {
            if let Err(e) = field.finalize() {
                return Err(format!(
                    "Erreur lors de la finalisation du champ '{}': {}",
                    name, e
                ));
            }
        }
        Ok(())
    }
    /// Valide le formulaire avec protection contre les appels récursifs
    /// Retourne un Result pour permettre la propagation des erreurs
    pub fn is_valid(&mut self) -> Result<bool, ValidationError> {
        // Protection contre les appels récursifs (ex: si clean() rappelle is_valid())
        VALIDATION_DEPTH.with(|depth| {
            let current = depth.get();
            if current > MAX_VALIDATION_DEPTH {
                return Err(ValidationError::StackOverflow);
            }
            depth.set(current + 1);
            let result = self.validate_fields();
            depth.set(current); // Restaure la profondeur
            result
        })
    }

    /// Validation interne des champs
    fn validate_fields(&mut self) -> Result<bool, ValidationError> {
        let mut is_all_valid = true;

        for field in self.fields.values_mut() {
            if field.required() && field.value().trim().is_empty() {
                field.set_error("Ce champ est obligatoire".to_string());
                is_all_valid = false;
                continue;
            }
            if !field.validate() {
                is_all_valid = false;
            }
        }

        let result = is_all_valid && self.global_errors.is_empty();

        if !result {
            if !self.global_errors.is_empty() {
                return Err(ValidationError::GlobalErrors(self.global_errors.clone()));
            } else {
                return Err(ValidationError::FieldValidation(self.errors()));
            }
        }

        Ok(true)
    }

    pub fn has_errors(&self) -> bool {
        !self.global_errors.is_empty() || self.fields.values().any(|f| f.error().is_some())
    }

    pub fn data(&self) -> JsonMap {
        self.fields
            .iter()
            .map(|(name, field)| (name.clone(), field.to_json_value()))
            .collect()
    }

    pub fn errors(&self) -> StrMap {
        let mut errs: StrMap = self
            .fields
            .iter()
            .filter_map(|(name, field)| field.error().map(|err| (name.clone(), err.clone())))
            .collect();

        if !self.global_errors.is_empty() {
            errs.insert("global".to_string(), self.global_errors.join(" | "));
        }
        errs
    }

    pub fn render(&self) -> Result<String, String> {
        let mut html = Vec::new();
        let tera_instance = self.tera.as_ref().ok_or("Tera non configuré")?;

        let js_html = self.render_js(tera_instance)?;

        if !js_html.is_empty() {
            html.push(js_html);
        }

        // 1. Render tous les fields
        for field in self.fields.values() {
            match field.render(tera_instance) {
                Ok(rendered) => html.push(rendered),
                Err(e) => return Err(format!("Erreur rendu '{}': {}", field.name(), e)),
            }
        }

        Ok(html.join("\n"))
    }

    pub fn get_value(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|field| field.value().to_string())
    }

    // ========================================================================
    // HELPERS DE CONVERSION TYPÉE
    // ========================================================================

    /// Retourne la valeur comme `String`, ou `String::new()` si le champ n'existe pas.
    pub fn get_string(&self, name: &str) -> String {
        self.get_value(name).unwrap_or_default()
    }

    /// Retourne la valeur comme `Option<String>`.
    /// `None` si le champ n'existe pas **ou** si la valeur est vide.
    pub fn get_option(&self, name: &str) -> Option<String> {
        self.get_value(name).filter(|v| !v.trim().is_empty())
    }

    /// Retourne la valeur comme `i32` (0 par défaut).
    pub fn get_i32(&self, name: &str) -> i32 {
        self.get_string(name).parse().unwrap_or(0)
    }

    /// Retourne la valeur comme `i64` (0 par défaut).
    pub fn get_i64(&self, name: &str) -> i64 {
        self.get_string(name).parse().unwrap_or(0)
    }

    /// Retourne la valeur comme `u32` (0 par défaut).
    pub fn get_u32(&self, name: &str) -> u32 {
        self.get_string(name).parse().unwrap_or(0)
    }

    /// Retourne la valeur comme `u64` (0 par défaut).
    pub fn get_u64(&self, name: &str) -> u64 {
        self.get_string(name).parse().unwrap_or(0)
    }

    /// Retourne la valeur comme `f32` (0.0 par défaut).
    pub fn get_f32(&self, name: &str) -> f32 {
        self.get_string(name)
            .replace(',', ".")
            .parse()
            .unwrap_or(0.0)
    }

    /// Retourne la valeur comme `f64` (0.0 par défaut).
    pub fn get_f64(&self, name: &str) -> f64 {
        self.get_string(name)
            .replace(',', ".")
            .parse()
            .unwrap_or(0.0)
    }

    /// Retourne la valeur comme `bool`.
    /// `true` si la valeur est `"true"`, `"1"` ou `"on"`.
    pub fn get_bool(&self, name: &str) -> bool {
        let val = self.get_string(name);
        matches!(val.as_str(), "true" | "1" | "on")
    }

    /// Retourne la valeur comme `Option<i32>`. `None` si vide.
    pub fn get_option_i32(&self, name: &str) -> Option<i32> {
        self.get_option(name)?.parse().ok()
    }

    /// Retourne la valeur comme `Option<i64>`. `None` si vide.
    pub fn get_option_i64(&self, name: &str) -> Option<i64> {
        self.get_option(name)?.parse().ok()
    }

    /// Retourne la valeur comme `Option<f64>`. `None` si vide.
    pub fn get_option_f64(&self, name: &str) -> Option<f64> {
        self.get_option(name)
            .and_then(|v| v.replace(',', ".").parse().ok())
    }

    /// Retourne la valeur comme `Option<bool>`. `None` si vide.
    pub fn get_option_bool(&self, name: &str) -> Option<bool> {
        self.get_option(name)
            .map(|v| matches!(v.as_str(), "true" | "1" | "on"))
    }
    // ── Date / Time ─────────────────────────────────────────────────────────────

    /// Retourne la valeur comme `NaiveDate` (format `YYYY-MM-DD`).
    /// Retourne `NaiveDate::default()` si le champ est vide ou invalide.
    pub fn get_naive_date(&self, name: &str) -> chrono::NaiveDate {
        self.get_option(name)
            .and_then(|v| chrono::NaiveDate::parse_from_str(&v, "%Y-%m-%d").ok())
            .unwrap_or_default()
    }

    /// Retourne la valeur comme `Option<NaiveDate>`. `None` si vide ou invalide.
    pub fn get_option_naive_date(&self, name: &str) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(&self.get_option(name)?, "%Y-%m-%d").ok()
    }

    /// Retourne la valeur comme `NaiveTime` (format `HH:MM`).
    /// Retourne `NaiveTime::default()` si le champ est vide ou invalide.
    pub fn get_naive_time(&self, name: &str) -> chrono::NaiveTime {
        self.get_option(name)
            .and_then(|v| chrono::NaiveTime::parse_from_str(&v, "%H:%M").ok())
            .unwrap_or_default()
    }

    /// Retourne la valeur comme `Option<NaiveTime>`. `None` si vide ou invalide.
    pub fn get_option_naive_time(&self, name: &str) -> Option<chrono::NaiveTime> {
        chrono::NaiveTime::parse_from_str(&self.get_option(name)?, "%H:%M").ok()
    }

    /// Retourne la valeur comme `NaiveDateTime` (format `YYYY-MM-DDTHH:MM`).
    /// Retourne `NaiveDateTime::default()` si le champ est vide ou invalide.
    pub fn get_naive_datetime(&self, name: &str) -> chrono::NaiveDateTime {
        self.get_option(name)
            .and_then(|v| chrono::NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M").ok())
            .unwrap_or_default()
    }

    /// Retourne la valeur comme `Option<NaiveDateTime>`. `None` si vide ou invalide.
    pub fn get_option_naive_datetime(&self, name: &str) -> Option<chrono::NaiveDateTime> {
        chrono::NaiveDateTime::parse_from_str(&self.get_option(name)?, "%Y-%m-%dT%H:%M").ok()
    }

    /// Retourne la valeur comme `DateTime<Utc>`. `Utc::now()` si vide ou invalide.
    pub fn get_datetime_utc(&self, name: &str) -> chrono::DateTime<chrono::Utc> {
        self.get_option(name)
            .and_then(|v| chrono::DateTime::parse_from_rfc3339(&v).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now)
    }

    /// Retourne la valeur comme `Option<DateTime<Utc>>`. `None` si vide ou invalide.
    pub fn get_option_datetime_utc(&self, name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(&self.get_option(name)?)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    // ── UUID ─────────────────────────────────────────────────────────────────────

    /// Retourne la valeur comme `Uuid`. `Uuid::nil()` si vide ou invalide.
    pub fn get_uuid(&self, name: &str) -> uuid::Uuid {
        self.get_option(name)
            .and_then(|v| uuid::Uuid::parse_str(&v).ok())
            .unwrap_or(uuid::Uuid::nil())
    }

    /// Retourne la valeur comme `Option<Uuid>`. `None` si vide ou invalide.
    pub fn get_option_uuid(&self, name: &str) -> Option<uuid::Uuid> {
        uuid::Uuid::parse_str(&self.get_option(name)?).ok()
    }
    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        let err_msg = db_err.to_string();

        // Gestion des erreurs d'unicité avec extraction automatique du champ
        if err_msg.contains("unique") || err_msg.contains("UNIQUE") || err_msg.contains("Duplicate")
        {
            let field_name = Self::extract_field_name(&err_msg);

            if let Some(field) = field_name {
                // Trouver le champ correspondant et lui attribuer l'erreur
                if let Some(form_field) = self.fields.get_mut(&field) {
                    let friendly_name = field.replace("_", " ");
                    form_field.set_error(format!("Ce {} est déjà utilisé.", friendly_name));
                } else {
                    // Si le champ n'existe pas dans le formulaire, erreur globale
                    self.global_errors
                        .push(format!("La valeur du champ '{}' est déjà utilisée.", field));
                }
            } else {
                // Erreur d'unicité mais impossible d'extraire le champ
                self.global_errors
                    .push("Une contrainte d'unicité a été violée.".to_string());
            }
        } else {
            // Autres erreurs de base de données
            self.global_errors.push(format!("Erreur DB: {}", err_msg));
        }
    }

    /// Extraire le nom du champ depuis différents formats d'erreur SQL
    fn extract_field_name(err_msg: &str) -> Option<String> {
        // 1. PostgreSQL: constraint name
        if let Some(cap) = CONSTRAINT_REGEX.captures(err_msg).ok()? {
            let constraint = cap.get(1)?.as_str();
            return Self::parse_constraint_name(constraint);
        }

        // 2. PostgreSQL: Key (field)=(value)
        if let Some(cap) = KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        // 3. SQLite: UNIQUE constraint failed: table.field
        if let Some(cap) = FAILED_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        // 4. MySQL: for key 'table.field'
        if let Some(cap) = FOR_KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        None
    }

    /// Parser le nom de contrainte pour extraire le nom du champ
    fn parse_constraint_name(constraint: &str) -> Option<String> {
        let parts: Vec<&str> = constraint.split('_').collect();

        if parts.len() >= 3 {
            // Format: table_field_key ou table_field_idx
            let field_parts = &parts[1..parts.len() - 1];
            return Some(field_parts.join("_"));
        }

        None
    }

    pub fn add_value(&mut self, name: &str, value: &str) {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
        }
    }
}
