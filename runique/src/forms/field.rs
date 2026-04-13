//! `RuniqueForm` trait: common interface for all Runique forms.
pub use crate::forms::{
    base::FormField, form::Forms, renderer::FormRenderer, validator::ValidationError,
};
use crate::utils::{
    aliases::{ATera, StrMap},
    trad::t,
};
use async_trait::async_trait;
use axum::http::Method;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

dyn_clone::clone_trait_object!(FormField);

/// Common logic for all `cleaned_*` — whitelist + priority: POST > path > query.
fn cleaned_value(form: &Forms, name: &str) -> Option<String> {
    if !form.fields.contains_key(name) {
        return None;
    }
    if let Some(val) = form.fields.get(name).map(|f| f.value()) {
        if !val.is_empty() {
            return Some(val.to_string());
        }
    }
    form.path_params
        .get(name)
        .or_else(|| form.query_params.get(name))
        .map(|s| s.to_string())
}

/// Main trait for typed forms with validation and saving
///
#[doc = include_str!("../../doc-tests/form/form_proc_macro.md")]
///
#[doc = include_str!("../../doc-tests/form/form_clean_save.md")]
#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    // ── Whitelisted access to values (POST > path param > query param) ──────────

    /// `String` — `None` if the field is unknown or empty.
    fn cleaned_string(&self, name: &str) -> Option<String> {
        cleaned_value(self.get_form(), name)
    }
    /// `i32` — `None` if unknown, empty, or not parseable.
    fn cleaned_i32(&self, name: &str) -> Option<i32> {
        cleaned_value(self.get_form(), name)?.parse().ok()
    }
    /// `i64` — `None` if unknown, empty, or not parseable.
    fn cleaned_i64(&self, name: &str) -> Option<i64> {
        cleaned_value(self.get_form(), name)?.parse().ok()
    }
    /// `u32` — `None` if unknown, empty, or not parseable.
    fn cleaned_u32(&self, name: &str) -> Option<u32> {
        cleaned_value(self.get_form(), name)?.parse().ok()
    }
    /// `u64` — `None` if unknown, empty, or not parseable.
    fn cleaned_u64(&self, name: &str) -> Option<u64> {
        cleaned_value(self.get_form(), name)?.parse().ok()
    }
    /// `f32` — handles `,` → `.`. `None` if unknown, empty, or not parseable.
    fn cleaned_f32(&self, name: &str) -> Option<f32> {
        cleaned_value(self.get_form(), name)?
            .replace(',', ".")
            .parse()
            .ok()
    }
    /// `f64` — handles `,` → `.`. `None` if unknown, empty, or not parseable.
    fn cleaned_f64(&self, name: &str) -> Option<f64> {
        cleaned_value(self.get_form(), name)?
            .replace(',', ".")
            .parse()
            .ok()
    }
    /// `bool` — `true` for `"true"`, `"1"`, `"on"` (case-insensitive).
    /// Returns `None` if the field does not exist in the form.
    /// Note: `fill()` normalizes unchecked checkboxes/radios to `"false"`,
    /// so this method always returns `Some(_)` for a submitted boolean field.
    fn cleaned_bool(&self, name: &str) -> Option<bool> {
        let v = cleaned_value(self.get_form(), name)?;
        Some(matches!(v.to_lowercase().as_str(), "true" | "1" | "on"))
    }

    /// Clears all form values (except CSRF).
    fn clear(&mut self) {
        self.get_form_mut().clear_values();
    }

    // Business validation hook for individual fields
    async fn clean_field(&mut self, name: &str) -> bool {
        self.get_form().fields.contains_key(name)
    }

    // Business validation hook for the entire form
    async fn clean(&mut self) -> Result<(), StrMap> {
        Ok(())
    }

    async fn is_valid(&mut self) -> bool {
        // If the form has no submitted data (e.g. first GET with no params), return false
        // without setting any field errors. This prevents showing validation errors on the
        // initial page load, and lets GET search forms fall through to their else branch
        // cleanly. Note: Forms::is_valid() (sync) does not have this guard.
        if !self.get_form().is_submitted() {
            return false;
        }

        let mut fields_valid = match self.get_form_mut().is_valid() {
            Ok(valid) => valid,
            Err(ValidationError::StackOverflow) => {
                self.get_form_mut()
                    .errors
                    .push(t("forms.validation_overflow").into_owned());
                return false;
            }
            Err(_) => return false,
        };

        let names: Vec<String> = self.get_form().fields.keys().cloned().collect();
        for name in names {
            if !self.clean_field(&name).await {
                fields_valid = false;
            }
        }

        if !fields_valid {
            return false;
        }

        match self.clean().await {
            Ok(_) => {
                if let Err(e) = self.get_form_mut().finalize() {
                    self.get_form_mut().errors.push(e);
                    return false;
                }
                true
            }
            Err(business_errors) => {
                let form = self.get_form_mut();
                for (name, msg) in business_errors {
                    if let Some(field) = form.fields.get_mut(&name) {
                        field.set_error(msg);
                    } else {
                        form.errors.push(msg);
                    }
                }
                false
            }
        }
    }

    /// Default atomic wrapper: opens a transaction and calls `save_txn`.
    ///
    /// - if `save_txn` returns Err -> automatic rollback
    /// - otherwise -> commit
    async fn save_txn(&mut self, _txn: &DatabaseTransaction) -> Result<(), DbErr> {
        Ok(())
    }

    /// Atomic wrapper: explicit transaction (avoids the 'static futures trap)
    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        match self.save_txn(&txn).await {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                // Try rollback, but return the original business/DB error.
                let _ = txn.rollback().await;
                Err(e)
            }
        }
    }

    fn database_error(&mut self, err: &DbErr) {
        self.get_form_mut().database_error(err);
    }

    fn build(tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);
        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    async fn build_with_data(
        raw_data: &StrMap,
        tera: ATera,
        csrf_token: &str,
        method: Method,
    ) -> Self {
        let mut form = Forms::new(csrf_token);

        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);

        Self::register_fields(&mut form);
        form.fill(raw_data, method);
        Self::from_form(form)
    }
}
