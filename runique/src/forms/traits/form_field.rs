//! `FormField`: what every form field implements (validation, rendering, the
//! value its column stores), on top of its `FieldConfig`.
use crate::{
    forms::{
        config::{BoolChoice, FieldBounds, FieldConfig},
        fields::{FileSize, file::is_staged_upload},
    },
    translation::{t, tf},
    utils::aliases::ATera,
};
use async_trait::async_trait;
use dyn_clone::DynClone;
use sea_orm::sea_query::ColumnType;
use serde_json::{Map, Value, json};
use tera::Context;

/// Common trait to access a field's configuration
pub trait FieldAccess {
    fn get_field_config(&self) -> &FieldConfig;
    fn get_field_config_mut(&mut self) -> &mut FieldConfig;

    /// `true` if the field's value must never be exposed.
    ///
    /// The framework's single point of truth on this: rendering, filling,
    /// logging and auditing all go through here instead of each comparing
    /// `field_type()` to the string `"password"` on their own.
    fn is_password(&self) -> bool {
        self.get_field_config().is_password()
    }
}

impl FieldAccess for FieldConfig {
    fn get_field_config(&self) -> &FieldConfig {
        self
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        self
    }
}

/// A file part sent under a text field's name: its value is the path the upload
/// was staged at, which a re-rendered form would show back.
pub(crate) fn is_staged_path(value: &str) -> bool {
    value.contains(".staging-") && value.split(',').any(|part| is_staged_upload(part.trim()))
}

/// Behavior every form field widget implements: validation, HTML rendering,
/// and JSON serialization for client-side use. Most getters/setters have
/// default implementations built on [`FieldAccess`]; only
/// [`FormField::validate`] and [`FormField::render`] are field-type specific
/// and have no default.
#[async_trait]
pub trait FormField: FieldAccess + DynClone + std::fmt::Debug + Send + Sync {
    // ========================================================================
    // GETTERS - Default implementation via FieldAccess
    // ========================================================================

    fn name(&self) -> &str {
        &self.get_field_config().name
    }

    fn label(&self) -> &str {
        &self.get_field_config().label
    }

    fn value(&self) -> &str {
        &self.get_field_config().value
    }

    fn placeholder(&self) -> &str {
        &self.get_field_config().placeholder
    }

    fn field_type(&self) -> &str {
        &self.get_field_config().type_field
    }

    fn template_name(&self) -> &str {
        &self.get_field_config().template_name
    }

    fn error(&self) -> Option<&String> {
        self.get_field_config().error.as_ref()
    }

    fn required(&self) -> bool {
        self.get_field_config().is_required.choice
    }

    // ========================================================================
    // SETTERS - Default implementation via FieldAccess
    // ========================================================================

    fn set_name(&mut self, name: &str) {
        self.get_field_config_mut().name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.get_field_config_mut().label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.get_field_config_mut().value = value.to_string();
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        self.get_field_config_mut().placeholder = placeholder.to_string();
    }

    fn set_error(&mut self, message: String) {
        let config = self.get_field_config_mut();
        config.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    /// Resets the field error (explicit equivalent of `set_error("")`)
    fn clear_error(&mut self) {
        self.get_field_config_mut().error = None;
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.get_field_config_mut().is_required =
            BoolChoice::new(required, msg.map(str::to_string));
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.get_field_config_mut()
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        self.get_field_config_mut().readonly = BoolChoice::new(readonly, msg.map(str::to_string));
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        self.get_field_config_mut().disabled = BoolChoice::new(disabled, msg.map(str::to_string));
    }

    /// Sets a value that came from the request (`Forms::fill`), as opposed to
    /// one set by code. Same as `set_value` unless the field has to check where
    /// the value comes from — a file field only takes a path to a file this
    /// request uploaded.
    fn set_submitted_value(&mut self, value: &str) {
        if is_staged_path(value) {
            tracing::warn!(field = %self.get_field_config().name, "a file was sent for a field that takes none, ignored");
            return;
        }
        self.set_value(value);
    }

    /// Holds an integer field to what its Rust column type can store. `false`
    /// if this isn't an integer field.
    fn set_type_bounds(&mut self, _min: i128, _max: i128) -> bool {
        false
    }

    /// The lengths or values this field accepts; none by default.
    fn bounds(&self) -> FieldBounds {
        FieldBounds::default()
    }

    /// Lowers the accepted upload size to the column's, if the field allows more.
    fn cap_max_size(&mut self, _bytes: u64) {}

    /// Model-defined ceiling for max_size (file fields only). None for other field types.
    fn model_max_size(&self) -> Option<u64> {
        None
    }

    /// Overrides the effective max_size. Returns Err if it exceeds the model ceiling.
    fn set_max_size_bounded(&mut self, _size: FileSize) -> Result<(), String> {
        Err("ce champ ne supporte pas max_size".to_string())
    }

    /// The column this field is stored in, once given one.
    fn column_type(&self) -> Option<&ColumnType> {
        self.get_field_config().column_type.as_ref()
    }

    /// Whether this field can store `column` at all ([`crate::forms::generic::accepts`]).
    /// `false` by default: only a field registered through `GenericField` is
    /// known, any other is refused rather than trusted.
    fn accepts_column(&self, _column: &ColumnType) -> bool {
        false
    }

    /// Gives the field its column: what it stores, and what it then holds the
    /// input to. Never loosens a bound already set. The caller has checked the
    /// field can store that column at all (`accepts`).
    fn apply_column_type(&mut self, column: &ColumnType) {
        self.get_field_config_mut().column_type = Some(column.clone());
    }

    /// The cleaned value, as the field's column stores it: `Ok(None)` when
    /// empty, or for a field that fills no column. No default: a field type
    /// says how it converts, or it doesn't compile.
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String>;

    /// An empty valude leaves the column as it is instead of clearing it: a
    /// password not retyped, a file not re-uploaded.
    fn keeps_when_empty(&self) -> bool {
        self.is_password()
    }

    /// Readonly or disabled: shown, never written. A disabled field isn't
    /// sent, so it would arrive empty; a readonly one is sent, so it can be
    /// forged.
    fn is_locked(&self) -> bool {
        let config = self.get_field_config();
        config.readonly.choice || config.disabled.choice
    }

    /// Field-type specific validation
    async fn validate(&mut self) -> bool;

    /// Rendering context shared by **every** field.
    ///
    /// Every [`FormField::render`] implementation starts from this and only adds
    /// its own variables. The `field_html/` templates all consume `field`,
    /// `readonly.choice` and `disabled.choice`: forgetting them was invisible
    /// under Tera 1, which evaluated a missing variable as false — the attribute
    /// was then simply never emitted, with no error at all. Nine renders out of
    /// eighteen were in that state.
    fn base_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("field", self.get_field_config());
        // Left out when unnamed: templates read `form.name` unconditionally,
        // so Tera fails the render of a form `try_form` never named.
        if let Some(name) = self.get_field_config().form_name {
            context.insert("form", &json!({ "name": name }));
        }
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());
        context
    }

    /// HTML rendering of the field
    fn render(&self, tera: &ATera) -> Result<String, String>;

    /// Renders the field's template with `context`; a failure names the template.
    #[doc(hidden)]
    fn render_template(&self, tera: &ATera, context: &Context) -> Result<String, String> {
        let template = &self.get_field_config().template_name;
        tera.render(template, context).map_err(|e| {
            tf(
                runique_i18n::keys::forms::error::FINALIZE,
                &[template, &e.to_string()],
            )
        })
    }

    /// Required and left empty: sets the field's own message, or `default_key`
    /// translated, and returns `true`.
    #[doc(hidden)]
    fn fails_required(&mut self, default_key: &str) -> bool {
        let config = self.get_field_config();
        if !config.is_required.choice || !config.value.trim().is_empty() {
            return false;
        }
        let msg = config
            .is_required
            .message
            .clone()
            .unwrap_or_else(|| t(default_key).to_string());
        self.set_error(msg);
        true
    }

    /// Finalization (e.g., password hashing)
    async fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Whether the field is saving an upload and removes the stored file it
    /// takes the place of (a file field declared with `.replace()`): the
    /// update then has to tell it which file that is (`replaced_file`).
    fn awaits_replaced_file(&self) -> bool {
        false
    }

    /// The value the field's column held before the update that is writing a
    /// new one. A field that replaces removes that file once the write is
    /// committed; any other ignores it.
    fn replaced_file(&mut self, _stored: &str) {}

    /// Publishes external resources prepared by `finalize`, between the form's
    /// database write and its commit.
    async fn publish_uploads(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Removes any resources published by `publish_uploads` when the write
    /// doesn't commit.
    async fn rollback_uploads(&mut self) {}

    /// Performs post-write cleanup after the database write succeeds.
    async fn complete_uploads(&mut self) {}

    fn to_json_value(&self) -> Value {
        json!(self.get_field_config().value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.get_field_config().is_required)
    }

    fn to_json_readonly(&self) -> Value {
        json!(self.get_field_config().readonly)
    }

    fn to_json_disabled(&self) -> Value {
        json!(self.get_field_config().disabled)
    }

    fn to_json_attributes(&self) -> Value {
        let attrs: Vec<(&String, &String)> =
            self.get_field_config().html_attributes.iter().collect();
        let map: Map<String, Value> = attrs
            .into_iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        Value::Object(map)
    }

    fn to_json_meta(&self) -> Value {
        json!({})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forms::{
        fields::{MediaField, TextField},
        generic::GenericField,
    };

    #[test]
    fn a_field_neither_readonly_nor_disabled_is_not_locked() {
        assert!(!TextField::text("t").is_locked());
    }

    #[test]
    fn a_readonly_field_is_locked() {
        let mut field = TextField::text("t");
        field.set_readonly(true, None);
        assert!(field.is_locked());
    }

    #[test]
    fn a_disabled_field_is_locked() {
        let mut field = TextField::text("t");
        field.set_disabled(true, None);
        assert!(field.is_locked());
    }

    #[test]
    fn a_field_readonly_false_is_not_locked() {
        let mut field = TextField::text("t");
        field.set_readonly(false, None);
        field.set_disabled(false, None);
        assert!(!field.is_locked());
    }

    #[test]
    fn readonly_does_not_hide_disabled_and_the_reverse() {
        let mut field = TextField::text("t");
        field.set_readonly(false, None);
        field.set_disabled(true, None);
        assert!(field.is_locked());
        let mut field = TextField::text("t");
        field.set_readonly(true, None);
        field.set_disabled(false, None);
        assert!(field.is_locked());
    }

    // The lock is the typed flag: nothing left in `extra_context` decides it.
    #[test]
    fn extra_context_neither_locks_nor_unlocks_a_field() {
        let mut field = TextField::text("t");
        field
            .base
            .extra_context
            .insert("readonly".into(), json!({"choice": true}));
        assert!(!field.is_locked());
        field.set_disabled(true, Some("why"));
        field
            .base
            .extra_context
            .insert("disabled".into(), json!({"choice": false}));
        assert!(field.is_locked());
        assert_eq!(field.base.disabled.message.as_deref(), Some("why"));
        assert_eq!(
            field.to_json_disabled(),
            json!({"choice": true, "message": "why"})
        );
    }

    #[test]
    fn a_locked_state_survives_the_generic_wrapper() {
        let mut field: GenericField = TextField::text("t").into();
        assert!(!field.is_locked());
        field.set_readonly(true, None);
        assert!(field.is_locked());
    }

    #[test]
    fn a_text_field_does_not_keep_its_column_when_empty() {
        assert!(!TextField::text("t").keeps_when_empty());
    }

    #[test]
    fn a_password_keeps_its_column_when_empty() {
        assert!(TextField::password("p").keeps_when_empty());
    }

    #[test]
    fn a_file_keeps_its_column_when_empty() {
        assert!(MediaField::image("f").keeps_when_empty());
    }

    // A form stores its fields wrapped in a `GenericField`.
    #[test]
    fn the_generic_wrapper_keeps_the_answer_of_the_field_it_wraps() {
        let file: GenericField = MediaField::image("f").into();
        assert!(file.keeps_when_empty());
        let password: GenericField = TextField::password("p").into();
        assert!(password.keeps_when_empty());
        let text: GenericField = TextField::text("t").into();
        assert!(!text.keeps_when_empty());
    }
}
