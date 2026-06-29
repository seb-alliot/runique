//! `Forms` — main form container: fields, validation, rendering, CSRF management.
use crate::forms::{
    base::FormField,
    fields::{ChoiceField, HiddenField, HoneypotField},
    generic::GenericField,
    renderer::FormRenderer,
    validator::{FormValidator, ValidationError},
};

use crate::middleware::errors::error::html_escape;
use crate::utils::config::TraceResult;
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
    /// Set to true by anti-bot middleware when honeypot field was filled.
    pub(crate) force_invalid: bool,
    /// Honeypot field name injected by anti-bot middleware (for rendering).
    pub(crate) honeypot_field_name: Option<String>,
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
        let mut state = serializer.serialize_struct("Forms", 9)?;

        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("form_errors", &self.errors)?;

        let js_files = self
            .renderer
            .as_ref()
            .map(|r| r.js_files.clone())
            .unwrap_or_default();
        state.serialize_field("js_files", &js_files)?;

        // Pre-rendered <script> block (real CSP nonce + resolved static URLs) from
        // the canonical `js.html` template. The per-field form filter reuses this
        // instead of re-emitting raw `{% csp %}` / `{% static %}` tags, which the
        // load-time preprocessor never sees and `| safe` would ship verbatim.
        let rendered_js = self
            .renderer
            .as_ref()
            .map(|r| r.render_js().unwrap_or_default())
            .unwrap_or_default();
        state.serialize_field("rendered_js", &rendered_js)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .forms
                    .as_ref()
                    .and_then(|f| f.render)
                {
                    crate::runique_log!(level, error = %e, "form render failed");
                }
                format!("<p style='color:red'>Render error: {}</p>", html_escape(&e))
            }
        };
        state.serialize_field("html", &rendered_html)?;

        let rendered_fields: HashMap<String, String> = match &self.renderer {
            Some(renderer) => self
                .fields
                .iter()
                .filter_map(|(name, field)| {
                    renderer
                        .render_field(field.as_ref())
                        .trace(
                            crate::utils::runique_log::get_log()
                                .forms
                                .as_ref()
                                .and_then(|f| f.render),
                            "field render failed",
                        )
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

        let honeypot_html = match (&self.honeypot_field_name, &self.renderer) {
            (Some(name), Some(renderer)) => {
                let hp = HoneypotField::new(name);
                renderer.render_field(&hp).trace_or_default(
                    crate::utils::runique_log::get_log()
                        .forms
                        .as_ref()
                        .and_then(|f| f.render),
                    "honeypot render",
                )
            }
            _ => String::new(),
        };
        state.serialize_field("honeypot_html", &honeypot_html)?;

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

    /// Creates a `Forms` container pre-loaded with a CSRF hidden field set to `csrf_token`.
    /// The CSRF token is validated upstream by the Prisme pipeline — `set_expected_value` is not called here.
    pub fn new(csrf_token: &str) -> Self {
        let mut fields: FieldsMap = IndexMap::new();
        let mut csrf_field = HiddenField::new_csrf();
        csrf_field.set_value(csrf_token);
        // CSRF is already validated upstream by the Prisme pipeline (extractor::check_csrf).
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
            force_invalid: false,
            honeypot_field_name: None,
        }
    }

    /// Registers the honeypot field name injected by the anti-bot middleware.
    pub fn set_honeypot(&mut self, name: &str) {
        self.honeypot_field_name = Some(name.to_string());
    }

    /// Injects path and query parameters so `cleaned_*` methods can read them (GET search forms).
    pub fn set_url_params(
        &mut self,
        path: &HashMap<String, String>,
        query: &HashMap<String, String>,
    ) {
        self.path_params = path.clone();
        self.query_params = query.clone();
    }

    /// Attaches the Tera-based HTML renderer. Required for `form.html` in templates.
    pub fn set_renderer(&mut self, renderer: FormRenderer) {
        self.renderer = Some(renderer);
    }

    /// Passes the CSP nonce to the renderer so script tags carry the correct `nonce` attribute.
    pub fn set_csp_nonce(&mut self, nonce: &str) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_nonce(nonce);
        }
    }

    /// Appends JavaScript file paths to the renderer's `js_files` list (injected via `form.js_files`).
    pub fn add_js(&mut self, files: &[&str]) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.add_js(files);
        }
    }

    /// Registers a pre-built `GenericField`. Prefer [`field`](Forms::field) for typed fields.
    pub fn field_generic(&mut self, field: GenericField) {
        self.fields
            .insert(field.name().to_string(), Box::new(field));
    }

    /// Replaces a field with a `ChoiceField` populated with the given options.
    /// Preserves the current value (pre-selects the matching option) and the required flag.
    /// Used by the admin daemon to inject FK select options loaded from the DB.
    pub fn field_choices(&mut self, name: &str, label: &str, choices: Vec<(String, String)>) {
        let current_value = self
            .fields
            .get(name)
            .map(|f| f.value().to_string())
            .unwrap_or_default();
        let required = self.fields.get(name).map(|f| f.required()).unwrap_or(false);
        let mut cf = ChoiceField::new(name).label(label);
        if required {
            cf = cf.required();
        }
        for (val, lbl) in choices {
            cf = cf.add_choice(&val, &lbl);
        }
        if !current_value.is_empty() {
            cf.base.value = current_value;
        }
        self.field_generic(cf.into());
    }

    /// Registers a typed field (e.g. `TextField`, `NumericField`, `FileField`, …).
    /// Called inside `RuniqueForm::register_fields`.
    pub fn field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + Into<GenericField> + 'static,
    {
        let generic_instance: GenericField = field_template.clone().into();
        if let Some(level) = crate::utils::runique_log::get_log()
            .forms
            .as_ref()
            .and_then(|f| f.field)
        {
            crate::runique_log!(
                level,
                field = %generic_instance.name(),
                kind = %generic_instance.field_type(),
                required = generic_instance.required(),
                "field registered"
            );
        }
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
                if let Some(level) = crate::utils::runique_log::get_log()
                    .forms
                    .as_ref()
                    .and_then(|f| f.set_value)
                {
                    if field.field_type() == "password" {
                        crate::runique_log!(level, field = %field.name(), "set_value [hidden]");
                    } else {
                        crate::runique_log!(level, field = %field.name(), value = %value, "set_value");
                    }
                }
                field.set_value(value);
            }
        }
        // Normalizes checkboxes/radios absent from POST → "false".
        // A browser does not send unchecked boxes: without this normalization a checkbox
        // with default=true would keep its "true" default even when unchecked.
        if allow_password {
            for field in self.fields.values_mut() {
                if matches!(field.field_type(), "checkbox" | "radio")
                    && !data.contains_key(field.name())
                {
                    if let Some(level) = crate::utils::runique_log::get_log()
                        .forms
                        .as_ref()
                        .and_then(|f| f.set_value)
                    {
                        crate::runique_log!(level, field = %field.name(), value = "false", "set_value [checkbox absent → false]");
                    }
                    field.set_value("false");
                }
            }
        }

        // A POST/PUT/PATCH is always a submission, even if all fields are empty.
        // A GET with query params is only "submitted" if at least one param is non-empty.
        self.submitted = allow_password || has_data;
    }

    // ── Field display overrides ──────────────────────────────────────────────

    /// Overrides the label of a registered field by name.
    pub fn field_label(&mut self, name: &str, label: &str) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_label(label);
        }
        self
    }

    /// Overrides the placeholder of a registered field by name.
    pub fn field_placeholder(&mut self, name: &str, placeholder: &str) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_placeholder(placeholder);
        }
        self
    }

    /// Overrides the required constraint of a registered field by name.
    pub fn field_required(&mut self, name: &str, required: bool) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_required(required, None);
        }
        self
    }

    /// Sets a registered field as read-only by name.
    pub fn field_readonly(&mut self, name: &str, readonly: bool) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_readonly(readonly, None);
        }
        self
    }

    /// Sets a registered field as disabled by name.
    pub fn field_disabled(&mut self, name: &str, disabled: bool) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_disabled(disabled, None);
        }
        self
    }

    /// Sets an arbitrary HTML attribute on a registered field by name.
    pub fn field_attr(&mut self, name: &str, key: &str, value: &str) -> &mut Self {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_html_attribute(key, value);
        }
        self
    }

    /// Overrides max_size for a file field. Returns Err if it exceeds the model ceiling.
    pub fn field_max_size(
        &mut self,
        name: &str,
        size: crate::forms::fields::FileSize,
    ) -> Result<&mut Self, String> {
        if let Some(f) = self.fields.get_mut(name) {
            f.set_max_size_bounded(size)?;
        }
        Ok(self)
    }

    /// Forces a value on a named field, bypassing `fill()`.
    /// Also marks the form as submitted if the value is non-empty.
    /// Use this for fields skipped by `fill()` (e.g. password hash pre-computed before POST parsing).
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
        let log_finalize = crate::utils::runique_log::get_log()
            .forms
            .as_ref()
            .and_then(|f| f.finalize);
        for (name, field) in self.fields.iter_mut() {
            match field.finalize() {
                Ok(()) => {
                    if let Some(level) = log_finalize {
                        crate::runique_log!(level, field = %name, kind = %field.field_type(), "finalize ok");
                    }
                }
                Err(e) => {
                    if let Some(level) = log_finalize {
                        crate::runique_log!(level, field = %name, kind = %field.field_type(), error = %e, "finalize error");
                    }
                    return Err(tf("forms.finalize_error", &[name, &e]));
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// VALIDATION (delegated to validator)
// ============================================================================

impl Forms {
    /// Synchronous validation — runs all field validators in order.
    /// Returns `Ok(false)` if the form was force-invalidated (honeypot).
    /// Used internally by [`RuniqueForm::is_valid`].
    pub fn is_valid(&mut self) -> Result<bool, ValidationError> {
        if self.force_invalid {
            return Ok(false);
        }
        self.validated = true;
        Self::validate(&mut self.fields, &self.errors)
    }
    /// Returns `true` if any field or form-level error is present.
    pub fn has_errors(&self) -> bool {
        FormValidator::has_errors(&self.fields, &self.errors)
    }

    /// Returns true if save() is allowed: is_valid() was called and passed, no force_invalid.
    pub(crate) fn is_save_allowed(&self) -> bool {
        !self.force_invalid && self.validated && !self.has_errors()
    }

    /// **Test-only.** Marks the form as validated without calling `is_valid()`.
    ///
    /// Use this in tests that verify save/hook behavior independently of field validation.
    /// Calling this in production code bypasses all validation — never use outside `#[cfg(test)]`.
    #[doc(hidden)]
    pub fn mark_validated(&mut self) {
        self.validated = true;
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
            // Only return early if parse succeeds; _pkey/_fkey fall through to other regexes.
            if let Some(field) = Self::parse_constraint_name(constraint) {
                return Some(field);
            }
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
        // PK and FK constraints are not field-level unique violations.
        if constraint.ends_with("_pkey") || constraint.ends_with("_fkey") {
            return None;
        }
        // PostgreSQL convention: tablename_fieldname_key
        // The field is the second-to-last segment (parts[-2] before "_key").
        // Taking parts[1..len-1] incorrectly includes the table name when it contains underscores.
        // Taking just parts[len-2] gives the last field segment — correct for single-word fields,
        // and a best-effort fallback for multi-word ones (handled upstream by KEY_REGEX / FAILED_REGEX).
        let parts: Vec<&str> = constraint.split('_').collect();
        if parts.len() >= 3 {
            return Some(parts[parts.len() - 2].to_string());
        }
        None
    }
}
