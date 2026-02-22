use crate::forms::base::FormField;
use crate::forms::fields::TextField;
use crate::forms::generic::GenericField;
use crate::forms::renderer::FormRenderer;
use crate::forms::validator::{FormValidator, ValidationError};
use crate::utils::aliases::{FieldsMap, StrMap};
use crate::utils::constante::CSRF_TOKEN_KEY;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Forms {
    pub fields: FieldsMap,
    pub global_errors: Vec<String>,
    pub session_csrf_token: String,
    renderer: Option<FormRenderer>,
}

impl std::fmt::Debug for Forms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forms")
            .field("fields_count", &self.fields.len())
            .field("has_renderer", &self.renderer.is_some())
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

        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;

        let js_files = self
            .renderer
            .as_ref()
            .map(|r| r.js_files.clone())
            .unwrap_or_default();
        state.serialize_field("js_files", &js_files)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Render error: {}</p>", e),
        };
        state.serialize_field("html", &rendered_html)?;

        let rendered_fields: HashMap<String, String> = match &self.renderer {
            Some(renderer) => self
                .fields
                .iter()
                .filter_map(|(name, field)| {
                    renderer
                        .render_field(field.as_ref())
                        .ok()
                        .map(|html| (name.clone(), html))
                })
                .collect(),
            None => HashMap::new(),
        };
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

// ============================================================================
// BUILD & CONFIGURATION
// ============================================================================
use std::cell::Cell;
const MAX_VALIDATION_DEPTH: usize = 10;

thread_local! {
    static VALIDATION_DEPTH: Cell<usize> = const { Cell::new(0) };
}
impl Forms {
    fn validate(fields: &mut FieldsMap, global_errors: &[String]) -> Result<bool, ValidationError> {
        VALIDATION_DEPTH.with(|depth| {
            let current = depth.get();
            if current > MAX_VALIDATION_DEPTH {
                return Err(ValidationError::StackOverflow);
            }
            depth.set(current + 1);
            let result = FormValidator::validate_fields(fields, global_errors);
            depth.set(current);
            result
        })
    }

    pub fn new(csrf_token: &str) -> Self {
        let mut fields: FieldsMap = IndexMap::new();
        let mut csrf_field = TextField::create_csrf();
        csrf_field.set_value(csrf_token);

        fields.insert(
            CSRF_TOKEN_KEY.to_string(),
            Box::new(csrf_field) as Box<dyn FormField>,
        );

        Self {
            fields,
            global_errors: Vec::new(),
            session_csrf_token: csrf_token.to_string(),
            renderer: None,
        }
    }

    pub fn set_renderer(&mut self, renderer: FormRenderer) {
        self.renderer = Some(renderer);
    }

    pub fn add_js(&mut self, files: &[&str]) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.add_js(files);
        }
    }

    pub fn field_generic(&mut self, field: GenericField) {
        self.fields
            .insert(field.name().to_string(), Box::new(field));
    }

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

    pub fn fill(&mut self, data: &StrMap) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }

    pub fn add_value(&mut self, name: &str, value: &str) {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
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
}

// ============================================================================
// VALIDATION (délégation au validator)
// ============================================================================

impl Forms {
    pub fn is_valid(&mut self) -> Result<bool, ValidationError> {
        Self::validate(&mut self.fields, &self.global_errors)
    }
    pub fn has_errors(&self) -> bool {
        FormValidator::has_errors(&self.fields, &self.global_errors)
    }

    pub fn errors(&self) -> StrMap {
        FormValidator::collect_errors(&self.fields, &self.global_errors)
    }
}

// ============================================================================
// RENDU (délégation au renderer)
// ============================================================================

impl Forms {
    pub fn render(&self) -> Result<String, String> {
        self.renderer
            .as_ref()
            .ok_or("Renderer non configuré")?
            .render(&self.fields)
    }
}

// ============================================================================
// EXTRACTION DE DONNÉES (Logique métier pure)
// ============================================================================

impl Forms {
    pub fn get_value(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|field| field.value().to_string())
    }
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
}

// ============================================================================
// GESTION DES ERREURS DB (Logique métier spécifique)
// ============================================================================

impl Forms {
    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        let err_msg = db_err.to_string();

        if err_msg.contains("unique") || err_msg.contains("UNIQUE") || err_msg.contains("Duplicate")
        {
            if let Some(field) = Self::extract_field_name(&err_msg) {
                if let Some(form_field) = self.fields.get_mut(&field) {
                    let friendly_name = field.replace("_", " ");
                    form_field.set_error(format!("Ce {} est déjà utilisé.", friendly_name));
                } else {
                    self.global_errors
                        .push(format!("La valeur du champ '{}' est déjà utilisée.", field));
                }
            } else {
                self.global_errors
                    .push("Une contrainte d'unicité a été violée.".to_string());
            }
        } else {
            self.global_errors.push(format!("Erreur DB: {}", err_msg));
        }
    }

    fn extract_field_name(err_msg: &str) -> Option<String> {
        use crate::utils::constante::{CONSTRAINT_REGEX, FAILED_REGEX, FOR_KEY_REGEX, KEY_REGEX};

        if let Some(cap) = CONSTRAINT_REGEX.captures(err_msg).ok()? {
            let constraint = cap.get(1)?.as_str();
            return Self::parse_constraint_name(constraint);
        }

        if let Some(cap) = KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        if let Some(cap) = FAILED_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        if let Some(cap) = FOR_KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        None
    }

    fn parse_constraint_name(constraint: &str) -> Option<String> {
        let parts: Vec<&str> = constraint.split('_').collect();
        if parts.len() >= 3 {
            let field_parts = &parts[1..parts.len() - 1];
            return Some(field_parts.join("_"));
        }
        None
    }
}
