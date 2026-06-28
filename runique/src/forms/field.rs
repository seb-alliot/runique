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

/// Context passed to `before_save` and `after_save` hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveContext {
    Create,
    Update,
    Delete,
}
dyn_clone::clone_trait_object!(FormField);

/// Common logic for all `cleaned_*` — whitelist + priority: POST > path > query.
fn cleaned_value(form: &Forms, name: &str) -> Option<String> {
    if !form.fields.contains_key(name) {
        return None;
    }
    if let Some(val) = form.fields.get(name).map(|f| f.value())
        && !val.is_empty()
    {
        return Some(val.to_string().trim().to_string());
    }
    form.path_params
        .get(name)
        .or_else(|| form.query_params.get(name))
        .map(|s| s.to_string())
}

/// Coerce a whitelisted value to `T`. Logs (field, raw, error) at `forms.field` level
/// when a value is present but fails to parse — instead of silently returning `None`.
/// An absent/empty value is not an error (returns `None` without logging).
fn coerce<T>(form: &Forms, name: &str) -> Option<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = cleaned_value(form, name)?;
    let parsed = raw.parse::<T>();
    log_coerce(name, &raw, parsed)
}

/// Logs (field, raw, error) at `forms.field` level when a present value fails to parse,
/// then returns `None`. Shared by `coerce` and the format-based `cleaned_*` helpers.
fn log_coerce<T, E: std::fmt::Display>(name: &str, raw: &str, res: Result<T, E>) -> Option<T> {
    match res {
        Ok(v) => Some(v),
        Err(e) => {
            if let Some(level) = crate::utils::config::runique_log::get_log()
                .forms
                .as_ref()
                .and_then(|f| f.field)
            {
                crate::runique_log!(
                    level,
                    field = %name,
                    raw = %raw,
                    error = %e,
                    "cleaned coercion failed"
                );
            }
            None
        }
    }
}

/// Rolls back a transaction and logs the rollback error instead of swallowing it.
/// A failed rollback leaves the DB in an undefined state — it must never be silent,
/// even though the caller returns the original business error.
async fn rollback_traced(txn: DatabaseTransaction) {
    if let Err(rb) = txn.rollback().await {
        tracing::warn!(error = %rb, "transaction rollback failed");
    }
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

    /// Hook to tweak the generated fields (label, placeholder, required, attrs…)
    /// right after they are registered. Default: no-op. Override it in your
    /// `impl RuniqueForm` block to customize a macro-generated form.
    fn customize(_form: &mut Forms) {}

    // ── Whitelisted access to values (POST > path param > query param) ──────────

    /// `String` — `None` if the field is unknown or empty.
    fn cleaned_string(&self, name: &str) -> Option<String> {
        cleaned_value(self.get_form(), name)
    }
    /// `i32` — `None` if unknown, empty, or not parseable.
    fn cleaned_i32(&self, name: &str) -> Option<i32> {
        coerce(self.get_form(), name)
    }
    /// `i64` — `None` if unknown, empty, or not parseable.
    fn cleaned_i64(&self, name: &str) -> Option<i64> {
        coerce(self.get_form(), name)
    }
    /// `u32` — `None` if unknown, empty, or not parseable.
    fn cleaned_u32(&self, name: &str) -> Option<u32> {
        coerce(self.get_form(), name)
    }
    /// `u64` — `None` if unknown, empty, or not parseable.
    fn cleaned_u64(&self, name: &str) -> Option<u64> {
        coerce(self.get_form(), name)
    }
    /// `f32` — handles `,` → `.`, parses via `Decimal` for precision. `None` if unknown, empty, or not parseable.
    fn cleaned_f32(&self, name: &str) -> Option<f32> {
        use rust_decimal::prelude::ToPrimitive;
        let raw = cleaned_value(self.get_form(), name)?;
        let v = raw.replace(',', ".");
        log_coerce(name, &raw, rust_decimal::Decimal::from_str_exact(&v))?.to_f32()
    }
    /// `f64` — handles `,` → `.`, parses via `Decimal` for precision. `None` if unknown, empty, or not parseable.
    fn cleaned_f64(&self, name: &str) -> Option<f64> {
        use rust_decimal::prelude::ToPrimitive;
        let raw = cleaned_value(self.get_form(), name)?;
        let v = raw.replace(',', ".");
        log_coerce(name, &raw, rust_decimal::Decimal::from_str_exact(&v))?.to_f64()
    }
    /// `bool` — `true` for `"true"`, `"1"`, `"on"` (case-insensitive).
    /// Returns `None` if the field does not exist in the form.
    /// Note: `fill()` normalizes unchecked checkboxes/radios to `"false"`,
    /// so this method always returns `Some(_)` for a submitted boolean field.
    fn cleaned_bool(&self, name: &str) -> Option<bool> {
        let v = cleaned_value(self.get_form(), name)?;
        Some(matches!(v.to_lowercase().as_str(), "true" | "1" | "on"))
    }

    /// `Uuid` — `None` if unknown, empty, or not parseable.
    fn cleaned_uuid(&self, name: &str) -> Option<uuid::Uuid> {
        coerce(self.get_form(), name)
    }

    /// `NaiveDate` — `None` if unknown, empty, or not parseable.
    fn cleaned_naive_date(&self, name: &str) -> Option<chrono::NaiveDate> {
        let raw = cleaned_value(self.get_form(), name)?;
        log_coerce(
            name,
            &raw,
            chrono::NaiveDate::parse_from_str(&raw, "%Y-%m-%d"),
        )
    }

    /// `NaiveTime` — `None` if unknown, empty, or not parseable.
    fn cleaned_naive_time(&self, name: &str) -> Option<chrono::NaiveTime> {
        let raw = cleaned_value(self.get_form(), name)?;
        log_coerce(name, &raw, chrono::NaiveTime::parse_from_str(&raw, "%H:%M"))
    }

    /// `NaiveDateTime` — `None` if unknown, empty, or not parseable.
    fn cleaned_naive_datetime(&self, name: &str) -> Option<chrono::NaiveDateTime> {
        let raw = cleaned_value(self.get_form(), name)?;
        log_coerce(
            name,
            &raw,
            chrono::NaiveDateTime::parse_from_str(&raw, "%Y-%m-%dT%H:%M"),
        )
    }

    /// `DateTime<Utc>` — `None` if unknown, empty, or not parseable.
    fn cleaned_datetime_utc(&self, name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        let raw = cleaned_value(self.get_form(), name)?;
        log_coerce(name, &raw, chrono::DateTime::parse_from_rfc3339(&raw))
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    /// SeaORM `ActiveEnum` — `None` if unknown, empty, or not a valid variant.
    fn cleaned_enum<T: sea_orm::ActiveEnum<Value = String>>(&self, name: &str) -> Option<T> {
        let raw = cleaned_value(self.get_form(), name)?;
        log_coerce(name, &raw, T::try_from_value(&raw))
    }

    // ── Field value overrides ───────────────────────────────────────────────

    /// Forces a value on a field, bypassing `fill()`. Useful for skipped fields (e.g. passwords).
    fn add_value(&mut self, name: &str, value: &str) -> &mut Self {
        self.get_form_mut().add_value(name, value);
        self
    }

    // ── Field display overrides ──────────────────────────────────────────────

    fn label(&mut self, name: &str, label: &str) -> &mut Self {
        self.get_form_mut().field_label(name, label);
        self
    }

    fn placeholder(&mut self, name: &str, placeholder: &str) -> &mut Self {
        self.get_form_mut().field_placeholder(name, placeholder);
        self
    }

    fn required(&mut self, name: &str, required: bool) -> &mut Self {
        self.get_form_mut().field_required(name, required);
        self
    }

    fn readonly(&mut self, name: &str, readonly: bool) -> &mut Self {
        self.get_form_mut().field_readonly(name, readonly);
        self
    }

    fn disabled(&mut self, name: &str, disabled: bool) -> &mut Self {
        self.get_form_mut().field_disabled(name, disabled);
        self
    }

    fn attr(&mut self, name: &str, key: &str, value: &str) -> &mut Self {
        self.get_form_mut().field_attr(name, key, value);
        self
    }

    /// Overrides max_size for a file field. Returns Err if it exceeds the model ceiling.
    fn max_size(
        &mut self,
        name: &str,
        size: crate::forms::fields::FileSize,
    ) -> Result<&mut Self, String> {
        self.get_form_mut().field_max_size(name, size)?;
        Ok(self)
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

    /// Default atomic wrapper: opens a transaction and calls `on_save`.
    ///
    /// - if `on_save` returns Err -> automatic rollback
    /// - otherwise -> commit
    async fn on_save(&mut self, _txn: &DatabaseTransaction) -> Result<(), DbErr> {
        Ok(())
    }

    /// Hook called inside the transaction, before `on_save`. Default: no-op.
    async fn before_save(
        &mut self,
        _ctx: SaveContext,
        _txn: &DatabaseTransaction,
    ) -> Result<(), DbErr> {
        Ok(())
    }

    /// Hook called inside the transaction, after `on_save`. Default: no-op.
    async fn after_save(
        &mut self,
        _ctx: SaveContext,
        _txn: &DatabaseTransaction,
    ) -> Result<(), DbErr> {
        Ok(())
    }

    /// Atomic wrapper: explicit transaction (avoids the 'static futures trap)
    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        if !self.get_form().is_save_allowed() {
            return Err(DbErr::Custom(
                "save() requires a successful is_valid() call — form not validated or invalid"
                    .to_string(),
            ));
        }
        let txn = db.begin().await?;

        match self.on_save(&txn).await {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                // Try rollback, but return the original business/DB error.
                rollback_traced(txn).await;
                Err(e)
            }
        }
    }

    /// Like `save`, but with explicit context for hooks.
    /// Order: `before_save` → `on_save` → `after_save` → commit.
    /// Any failure triggers rollback and returns the original error.
    async fn save_as(&mut self, ctx: SaveContext, db: &DatabaseConnection) -> Result<(), DbErr> {
        if !self.get_form().is_save_allowed() {
            return Err(DbErr::Custom(
                "save_as() requires a successful is_valid() call — form not validated or invalid"
                    .to_string(),
            ));
        }
        let txn = db.begin().await?;

        if let Err(e) = self.before_save(ctx, &txn).await {
            rollback_traced(txn).await;
            return Err(e);
        }
        if let Err(e) = self.on_save(&txn).await {
            rollback_traced(txn).await;
            return Err(e);
        }
        if let Err(e) = self.after_save(ctx, &txn).await {
            rollback_traced(txn).await;
            return Err(e);
        }

        txn.commit().await
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
