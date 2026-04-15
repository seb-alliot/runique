//! `Forms` — main form container: fields, validation, rendering, CSRF management.
use crate::forms::{
    base::FormField,
    fields::HiddenField,
    generic::GenericField,
    renderer::FormRenderer,
    validator::{FormValidator, ValidationError},
};

use crate::middleware::errors::error::html_escape;
use crate::utils::{
    aliases::{FieldsMap, StrMap},
    constante::session_key::session::CSRF_TOKEN_KEY,
    trad::{t, tf},
};
use axum::http::Method;
use indexmap::IndexMap;
use serde::{
    Serialize,
    ser::{SerializeStruct, Serializer},
};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Container of form fields with validation and HTML rendering
///
#[doc = include_str!("../../doc-tests/form/form_manual.md")]
#[derive(Clone)]
pub struct Forms {
    pub fields: FieldsMap,
    pub errors: Vec<String>,
    pub session_csrf_token: String,
    renderer: Option<FormRenderer>,
    submitted: bool,
    validated: bool,
    pub(crate) path_params: HashMap<String, String>,
    pub(crate) query_params: HashMap<String, String>,
}

impl std::fmt::Debug for Forms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forms")
            .field("fields_count", &self.fields.len())
            .field("has_renderer", &self.renderer.is_some())
            .field("errors", &self.errors)
            .finish()
    }
}

impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 8)?;

        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("form_errors", &self.errors)?;

        let js_files = self
            .renderer
            .as_ref()
            .map(|r| r.js_files.clone())
            .unwrap_or_default();
        state.serialize_field("js_files", &js_files)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Render error: {}</p>", html_escape(&e)),
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
    fn validate(fields: &mut FieldsMap, errors: &[String]) -> Result<bool, ValidationError> {
        VALIDATION_DEPTH.with(|depth| {
            let current = depth.get();
            if current > MAX_VALIDATION_DEPTH {
                return Err(ValidationError::StackOverflow);
            }
            depth.set(current.saturating_add(1));
            let result = FormValidator::validate_fields(fields, errors);
            depth.set(current);
            result
        })
    }

    pub fn new(csrf_token: &str) -> Self {
        let mut fields: FieldsMap = IndexMap::new();
        let mut csrf_field = HiddenField::new_csrf();
        csrf_field.set_value(csrf_token);
        // CSRF is already validated upstream by csrf_gate (Prisme).
        // set_expected_value is not called here: masked tokens are different on
        // each request (random mask), ct_eq would systematically fail and block
        // is_valid() on all forms.

        fields.insert(
            CSRF_TOKEN_KEY.to_string(),
            Box::new(csrf_field) as Box<dyn FormField>,
        );

        Self {
            fields,
            errors: Vec::new(),
            session_csrf_token: csrf_token.to_string(),
            renderer: None,
            submitted: false,
            validated: false,
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    pub fn set_url_params(
        &mut self,
        path: HashMap<String, String>,
        query: HashMap<String, String>,
    ) {
        self.path_params = path;
        self.query_params = query;
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

    /// Fills the form fields from a data map.
    /// If allow_password is false, password fields are ignored (GET security).
    /// In PATCH mode (admin edit), password fields have their required constraint
    /// relaxed: empty value = keep existing (NotSet DB side).
    pub fn fill(&mut self, data: &StrMap, method: Method) {
        let allow_password = matches!(method, Method::POST | Method::PUT | Method::PATCH);
        let is_edit = matches!(method, Method::PATCH | Method::PUT);
        let mut has_data = false;
        for field in self.fields.values_mut() {
            if field.field_type() == "password" && !allow_password {
                continue;
            }
            if field.field_type() == "password" && is_edit {
                field.set_required(false, None);
            }
            if let Some(value) = data.get(field.name()) {
                if !value.trim().is_empty() {
                    has_data = true;
                }
                field.set_value(value);
            }
        }
        // Normalizes checkboxes/radios absent from POST → "false".
        // A browser does not send unchecked boxes: without this normalization,
        // their value would remain "" and get_string().is_empty() would return true wrongly.
        if allow_password {
            for field in self.fields.values_mut() {
                if matches!(field.field_type(), "checkbox" | "radio") && field.value().is_empty() {
                    field.set_value("false");
                }
            }
        }

        // A POST/PUT/PATCH is always a submission, even if all fields are empty.
        // A GET with query params is only "submitted" if at least one param is non-empty.
        self.submitted = allow_password || has_data;
    }

    pub fn add_value(&mut self, name: &str, value: &str) {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
            if !value.trim().is_empty() && name != CSRF_TOKEN_KEY {
                self.submitted = true;
            }
        }
    }

    /// Clears all field values (except CSRF).
    /// To be called after reading cleaned data, before a redirect.
    pub fn clear_values(&mut self) {
        for (name, field) in self.fields.iter_mut() {
            if name != CSRF_TOKEN_KEY {
                field.set_value("");
            }
        }
        self.submitted = false;
        self.validated = false;
    }

    pub fn finalize(&mut self) -> Result<(), String> {
        for (name, field) in self.fields.iter_mut() {
            if let Err(e) = field.finalize() {
                return Err(tf("forms.finalize_error", &[name, &e]));
            }
        }
        Ok(())
    }
}

// ============================================================================
// VALIDATION (delegated to validator)
// ============================================================================

impl Forms {
    pub fn is_valid(&mut self) -> Result<bool, ValidationError> {
        // For unit tests and robustness, we always validate if is_valid() is called,
        // even if the form is not marked as submitted (e.g., no field filled).
        self.validated = true;
        Self::validate(&mut self.fields, &self.errors)
    }
    pub fn has_errors(&self) -> bool {
        FormValidator::has_errors(&self.fields, &self.errors)
    }

    pub fn errors(&self) -> StrMap {
        FormValidator::collect_errors(&self.fields, &self.errors)
    }
}

// ============================================================================
// RENDERING (delegated to renderer)
// ============================================================================

impl Forms {
    pub fn render(&self) -> Result<String, String> {
        self.renderer
            .as_ref()
            .ok_or_else(|| t("forms.tera_not_configured").into_owned())?
            .render(&self.fields, &self.errors)
    }
}

// ============================================================================
// DATA EXTRACTION (Pure business logic)
// ============================================================================

impl Forms {
    pub(crate) fn is_submitted(&self) -> bool {
        self.submitted
    }
}

// ============================================================================
// DB ERROR MANAGEMENT (Specific business logic)
// ============================================================================

impl Forms {
    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        let err_msg = db_err.to_string();

        if err_msg.contains("unique") || err_msg.contains("UNIQUE") || err_msg.contains("Duplicate")
        {
            if let Some(field) = Self::extract_field_name(&err_msg) {
                if let Some(form_field) = self.fields.get_mut(&field) {
                    let friendly_name = field.replace("_", " ");
                    form_field.set_error(tf("forms.unique_field_taken", &[&friendly_name]));
                } else {
                    self.errors.push(tf("forms.unique_value_taken", &[&field]));
                }
            } else {
                self.errors
                    .push(t("forms.unique_constraint_violated").into_owned());
            }
        } else {
            self.errors.push(tf("forms.db_error", &[&err_msg]));
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
            let field_parts = &parts[1..parts.len().saturating_sub(1)];
            return Some(field_parts.join("_"));
        }
        None
    }
}
