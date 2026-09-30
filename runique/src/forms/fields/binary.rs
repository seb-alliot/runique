//! Binary field: a file upload whose bytes are stored in the database column
//! (`binary`, `var_binary`, `blob`), instead of a path to a file on disk.
//!
//! Form values travel as strings up to the conversion into the entity, and
//! arbitrary bytes aren't valid UTF-8: `finalize` reads the staged upload and
//! hands its content on as base64, which [`decode_binary`] turns back into bytes.
use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::forms::fields::file::is_staged_upload;
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;

/// File input stored as bytes. Accepts one upload, up to `max_size` bytes.
#[derive(Clone, Serialize, Debug)]
pub struct BinaryField {
    pub base: FieldConfig,
    /// Largest accepted upload, in bytes — the column's length for
    /// `binary(n)` / `var_binary(n)`.
    pub max_size: Option<u64>,
}

impl BinaryField {
    /// Creates a binary upload field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "binary", "base_file.html"),
            max_size: None,
        }
    }

    /// Largest accepted upload, in bytes.
    pub fn max_size(mut self, bytes: u64) -> Self {
        self.max_size = Some(bytes);
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required (no upload fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

/// Bytes of a binary field's value, as `finalize` encoded them. `None` if the
/// value isn't valid base64.
pub fn decode_binary(value: &str) -> Option<Vec<u8>> {
    STANDARD.decode(value.trim()).ok()
}

impl CommonFieldConfig for BinaryField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

#[async_trait]
impl FormField for BinaryField {
    fn cap_max_size(&mut self, bytes: u64) {
        if self.max_size.is_none_or(|s| s > bytes) {
            self.max_size = Some(bytes);
        }
    }

    /// Only a file this request uploaded (one, staged by `parse_multipart`):
    /// a path sent as text would otherwise be read into the database.
    fn set_submitted_value(&mut self, value: &str) {
        let value = value.trim();
        if !value.is_empty() && !is_staged_upload(value) {
            tracing::warn!(field = %self.base.name, "binary field: submitted value is not a staged upload, ignored");
            return;
        }
        self.set_value(value);
    }

    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim().to_string();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.file_required").to_string());
            self.set_error(msg);
            return false;
        }

        if let Some(max) = self.max_size
            && is_staged_upload(&val)
            && let Ok(meta) = tokio::fs::metadata(&val).await
            && meta.len() > max
        {
            let mb = |bytes: u64| format!("{:.1}", bytes as f64 / (1024.0 * 1024.0));
            let name = std::path::Path::new(&val)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let _ = tokio::fs::remove_file(&val).await;
            self.base.value.clear();
            self.set_error(tf(
                "forms.file_too_large",
                &[name.as_str(), mb(meta.len()).as_str(), mb(max).as_str()],
            ));
            return false;
        }

        self.clear_error();
        true
    }

    /// Replaces the staged upload by its content, base64-encoded, and removes
    /// the staged file. A value that isn't a staged upload is left as it is.
    async fn finalize(&mut self) -> Result<(), String> {
        let val = self.base.value.trim().to_string();
        if !is_staged_upload(&val) {
            return Ok(());
        }
        let bytes = tokio::fs::read(&val)
            .await
            .map_err(|e| format!("read upload '{val}': {e}"))?;
        if let Err(e) = tokio::fs::remove_file(&val).await {
            tracing::warn!(path = %val, error = %e, "binary field: staged upload removal failed");
        }
        self.base.value = STANDARD.encode(bytes);
        Ok(())
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        // Bytes are never sent back into the page.
        let mut field = self.base.clone();
        field.value.clear();
        context.insert("field", &field);
        context.insert("is_file", &true);
        if let Some(size) = self.max_size {
            context.insert("max_size", &size);
            context.insert("max_size_mb", &(size as f64 / (1024.0 * 1024.0)));
        }
        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}
