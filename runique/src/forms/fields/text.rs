//! Champs texte : `TextField`, `EmailField`, `UrlField`, `TextAreaField`, `PasswordField`.
pub use crate::forms::generic::GenericField;
use crate::forms::{
    base::{FieldConfig, FormField, TextConfig},
    options::LengthConstraint,
};
use crate::utils::{
    aliases::ATera,
    password::{PasswordConfig, PasswordService},
    trad::{t, tf},
};
use async_trait::async_trait;
use serde::Serialize;
use validator::ValidateEmail;

/// Whether `value` is an absolute `http` / `https` URL. Read by the parser
/// browsers follow, so what it says is what an `href` would do:
/// `java<TAB>script:alert(1)` is `javascript:` to both, and refused.
fn is_web_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
}

/// Single-line text input. Construct with [`TextField::text`], [`::email`](TextField::email),
/// [`::password`](TextField::password), [`::url`](TextField::url), [`::phone`](TextField::phone),
/// [`::textarea`](TextField::textarea), or [`::richtext`](TextField::richtext).
#[derive(Clone, Serialize, Debug)]
pub struct TextField {
    pub base: FieldConfig,
    pub config: TextConfig,
    pub format: SpecialFormat,
    pub hash_password: bool, // true by default
    /// The value came from the request: a password is then hashed whatever it
    /// looks like. Only a value set by code may already be a hash.
    #[serde(skip)]
    submitted: bool,
}

/// Validation format applied to a [`TextField`].
#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum SpecialFormat {
    None,
    Email,
    Url,
    Password,
    RichText,
    Csrf,
    Phone,
}
field_base!(TextField, label);

// Implementation of methods for TextField
impl TextField {
    // Private generic constructor based on the generic field — avoids code duplication
    fn create(name: &str, type_field: &str, format: SpecialFormat) -> Self {
        Self {
            base: FieldConfig::new(name, type_field, "base_string.html"),
            config: TextConfig::default(),
            format,
            hash_password: true,
            submitted: false,
        }
    }
    /// Disables automatic password hashing (helper)
    pub fn no_hash(mut self) -> Self {
        self.hash_password = false;
        self
    }
    /// Minimum character count. `msg` overrides the default error message (pass `""` for default).
    pub fn min_length(mut self, min: u32, msg: &str) -> Self {
        self.config.min_length = Some(LengthConstraint {
            value: min,
            message: (!msg.is_empty()).then(|| msg.to_string()),
        });
        self
    }

    /// Maximum character count. `msg` overrides the default error message (pass `""` for default).
    pub fn max_length(mut self, max: u32, msg: &str) -> Self {
        self.config.max_length = Some(LengthConstraint {
            value: max,
            message: (!msg.is_empty()).then(|| msg.to_string()),
        });
        self
    }

    /// Plain text `<input type="text">`.
    pub fn text(name: &str) -> Self {
        Self::create(name, "text", SpecialFormat::None)
    }
    /// Multi-line `<textarea>`.
    pub fn textarea(name: &str) -> Self {
        Self::create(name, "textarea", SpecialFormat::None)
    }
    /// Rich-text editor (HTML sanitized on save).
    pub fn richtext(name: &str) -> Self {
        Self::create(name, "richtext", SpecialFormat::RichText)
    }
    /// Password input. Value is auto-hashed on `finalize()` when password mode is `Auto`.
    ///
    /// The protection doesn't depend on this constructor: `FieldConfig::new`
    /// marks every field of type `password`.
    pub fn password(name: &str) -> Self {
        Self::create(name, "password", SpecialFormat::Password)
    }
    /// Email input. Validates RFC format and normalizes to lowercase.
    pub fn email(name: &str) -> Self {
        let mut field = Self::create(name, "email", SpecialFormat::Email);
        field.base.value = field.base.value.to_lowercase();
        field
    }
    /// URL input. Accepts an absolute `http` or `https` URL only: the value
    /// can go into an `href`.
    pub fn url(name: &str) -> Self {
        Self::create(name, "url", SpecialFormat::Url)
    }
    /// Phone number input (`<input type="tel">`). Validates E.164-compatible format.
    pub fn phone(name: &str) -> Self {
        Self::create(name, "tel", SpecialFormat::Phone)
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the HTML `placeholder` attribute.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    /// Renders the field as read-only. `msg` is shown instead of the input when displayed.
    pub fn readonly(mut self, msg: &str) -> Self {
        self.set_readonly(true, Some(msg));
        self
    }

    /// Renders the field as disabled. `msg` is shown instead of the input when displayed.
    pub fn disabled(mut self, msg: &str) -> Self {
        self.set_disabled(true, Some(msg));
        self
    }

    /// Number of visible rows for `textarea` fields.
    pub fn rows(mut self, rows: usize) -> Self {
        self.base
            .extra_context
            .insert("rows".to_string(), serde_json::json!(rows));
        self
    }
}

#[async_trait]
impl FormField for TextField {
    /// A `VARCHAR(n)` column bounds the field to `n`, never above what it
    /// already allowed.
    fn apply_column_type(&mut self, column: &sea_orm::sea_query::ColumnType) {
        if let sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::N(n)) = column
            && self.config.max_length.as_ref().is_none_or(|m| m.value > *n)
        {
            self.config.max_length = Some(LengthConstraint {
                value: *n,
                message: None,
            });
        }
        self.base.column_type = Some(column.clone());
    }

    /// A password leaves only as the hash `finalize` made of it: read before
    /// validation, or from a `no_hash` field (a login check), it's `None`,
    /// never the text typed in.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        if self.base.is_password() {
            let hashed = self.hash_password
                && !self.submitted
                && PasswordService::new(crate::utils::password::password_get())
                    .is_already_hashed(&self.base.value);
            if !hashed {
                return Ok(None);
            }
        }
        crate::forms::column_value::read_field(
            &self.base,
            &self.base.value,
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    fn bounds(&self) -> crate::forms::base::FieldBounds {
        crate::forms::base::FieldBounds {
            min_length: self.config.min_length.as_ref().map(|l| l.value),
            max_length: self.config.max_length.as_ref().map(|l| l.value),
            ..Default::default()
        }
    }

    fn set_value(&mut self, value: &str) {
        let cleaned = match self.format {
            SpecialFormat::Password | SpecialFormat::Csrf => value.to_string(),
            SpecialFormat::RichText => crate::utils::sanitizer::sanitize_rich(value),
            // `sanitize_strict` already strips every tag (scripts included) via ammonia
            // and decodes entities, so legitimate `>`, `&`, `<` survive. The XSS guard
            // stays the output-side auto-escaping; no lossy input mutation needed.
            _ => crate::utils::sanitizer::sanitize_strict(value),
        };
        self.base.value = cleaned;
        self.submitted = false;
    }

    fn set_submitted_value(&mut self, value: &str) {
        if crate::forms::base::is_staged_path(value) {
            tracing::warn!(field = %self.base.name, "a file was sent for a field that takes none, ignored");
            return;
        }
        self.set_value(value);
        self.submitted = true;
    }
    async fn validate(&mut self) -> bool {
        if self.fails_required("forms.required") {
            return false;
        }
        // Initial trim
        let mut val = self.base.value.trim().to_string();

        if val.is_empty() {
            return true;
        }

        // Minimum length validation
        if let Some(limits) = &self.config.min_length {
            let count: u32 = val.chars().count() as u32;
            if count < limits.value {
                let msg = limits
                    .message
                    .clone()
                    .unwrap_or_else(|| tf("forms.too_short", &[&limits.value]));
                self.set_error(msg);
                return false;
            }
        }

        // Maximum length validation
        if let Some(limits) = &self.config.max_length {
            let count: u32 = val.chars().count() as u32;
            if count > limits.value {
                let msg = limits
                    .message
                    .clone()
                    .unwrap_or_else(|| tf("forms.too_long", &[&limits.value]));
                self.set_error(msg);
                return false;
            }
        }

        // Special format validation
        match &self.format {
            SpecialFormat::Email if !val.validate_email() => {
                self.set_error(t("forms.email_invalid").to_string());
                return false;
            }
            SpecialFormat::Email => {
                val = val.to_lowercase();
            }
            SpecialFormat::Url if !is_web_url(&val) => {
                self.set_error(t("forms.url_invalid").to_string());
                return false;
            }
            SpecialFormat::Phone => {
                let digit_count = val.chars().filter(|c| c.is_ascii_digit()).count();
                let has_only_valid_chars = val
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '+' | ' ' | '-' | '(' | ')'));
                // '+' autorisé uniquement en première position
                let plus_only_at_start = val.match_indices('+').all(|(i, _)| i == 0);
                if !has_only_valid_chars || !plus_only_at_start || !(7..=15).contains(&digit_count)
                {
                    self.set_error(t("forms.phone_invalid").to_string());
                    return false;
                }
            }
            _ => {}
        }

        // Update the cleaned value
        self.base.value = val;

        self.clear_error();
        true
    }

    async fn finalize(&mut self) -> Result<(), String> {
        if let SpecialFormat::Password = &self.format
            && self.hash_password
            && !self.base.value.is_empty()
        {
            let config = crate::utils::password::password_get();
            if let PasswordConfig::Auto(_) = config {
                let service = PasswordService::new(config);
                // A typed value is hashed even when it looks like a hash: stored
                // as is, it would carry cost parameters of the visitor's choosing.
                if self.submitted || !service.is_already_hashed(&self.base.value) {
                    self.base.value = service
                        .hash(&self.base.value)
                        .map_err(|e| tf("forms.hash_error", &[&e.to_string()]).to_string())?;
                    self.submitted = false;
                }
            }
        }
        Ok(())
    }
    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        // Prepare a "secured" version of the base.
        // Le vidage est piloté par `is_password` et non par `SpecialFormat`, pour
        // qu'un champ marqué sensible autrement qu'en `SpecialField::password`
        // bénéficie de la même règle.
        let mut base_data = self.base.clone();
        if base_data.is_password() {
            base_data.value = String::new();
        }

        // Écrase le `field` du contexte de base : la valeur d'un mot de passe ne
        // doit jamais repartir dans le HTML rendu.
        context.insert("field", &base_data);
        context.insert("input_type", &self.base.type_field);

        if let Some(l) = &self.config.min_length {
            context.insert("min_length", &l.value);
        }
        if let Some(l) = &self.config.max_length {
            context.insert("max_length", &l.value);
        }

        if let Some(rows) = self.base.extra_context.get("rows").and_then(|r| r.as_u64()) {
            context.insert("rows", &rows);
        }

        self.render_template(tera, &context)
    }
}
