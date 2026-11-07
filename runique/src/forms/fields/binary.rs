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
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::column_value::read_field(
            &self.base,
            &self.base.value,
            sea_orm::sea_query::ColumnType::Blob,
        )
    }

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

        if val.is_empty() {
            self.clear_error();
            return true;
        }

        // Refused when in doubt: an upload that can't be recognised or sized
        // would otherwise skip the column's limit and still be read by
        // `finalize`.
        if !is_staged_upload(&val) {
            tracing::warn!(field = %self.base.name, "binary field: value is no longer a staged upload, refused");
            self.base.value.clear();
            self.set_error(t("forms.file_unreadable").to_string());
            return false;
        }
        let size = match tokio::fs::metadata(&val).await {
            Ok(meta) => meta.len(),
            Err(e) => {
                tracing::warn!(field = %self.base.name, error = %e, "binary field: staged upload size unreadable, refused");
                self.base.value.clear();
                self.set_error(t("forms.file_unreadable").to_string());
                return false;
            }
        };

        if let Some(max) = self.max_size
            && size > max
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
                &[name.as_str(), mb(size).as_str(), mb(max).as_str()],
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
            .map_err(|e| super::media::upload_failed(format!("read upload '{val}': {e}")))?;
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

/// Written from cargo-mutants survivors (2026-10-02).
#[cfg(test)]
mod guarantees {
    use super::*;
    use std::fs;

    #[test]
    fn cap_max_size_only_ever_lowers_the_limit() {
        let mut field = BinaryField::new("blob").max_size(100);
        field.cap_max_size(200);
        assert_eq!(field.max_size, Some(100));
        field.cap_max_size(100);
        assert_eq!(field.max_size, Some(100));
        field.cap_max_size(40);
        assert_eq!(field.max_size, Some(40));
        let mut unbounded = BinaryField::new("blob");
        unbounded.cap_max_size(70);
        assert_eq!(unbounded.max_size, Some(70));
    }

    #[tokio::test]
    async fn an_upload_of_exactly_the_limit_is_accepted() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let root = std::env::temp_dir().join(format!("rq_bin_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        unsafe { std::env::set_var("MEDIA_ROOT", root.to_str().unwrap()) };

        let exact = staging.join("exact.bin");
        fs::write(&exact, [7u8; 8]).unwrap();
        let mut field = BinaryField::new("thumb").max_size(8);
        field.set_value(&exact.to_string_lossy());
        assert!(field.validate().await, "8 bytes in an 8-byte column");

        crate::config::static_files::restore_media_root();
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn an_upload_that_is_no_longer_recognised_is_refused_not_skipped() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let root = std::env::temp_dir().join(format!("rq_bin_{}", uuid::Uuid::new_v4()));
        let staging = root.join(format!(".staging-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&staging).unwrap();
        unsafe { std::env::set_var("MEDIA_ROOT", root.to_str().unwrap()) };

        // Outside the staging folders: the limit can't be checked.
        let outside = root.join("outside.bin");
        fs::write(&outside, [7u8; 20]).unwrap();
        let mut field = BinaryField::new("thumb").max_size(8);
        field.set_value(&outside.to_string_lossy());
        assert!(!field.validate().await, "not a staged upload");
        assert!(field.base.value.is_empty(), "finalize must not read it");

        // Staged, then gone before validation.
        let gone = staging.join("gone.bin");
        fs::write(&gone, [7u8; 4]).unwrap();
        let mut field = BinaryField::new("thumb").max_size(8);
        field.set_value(&gone.to_string_lossy());
        fs::remove_file(&gone).unwrap();
        assert!(!field.validate().await, "vanished upload");

        crate::config::static_files::restore_media_root();
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn no_upload_on_an_optional_field_is_valid() {
        let mut field = BinaryField::new("thumb").max_size(8);
        assert!(field.validate().await);
        let mut required = BinaryField::new("thumb").required();
        assert!(!required.validate().await);
    }

    #[test]
    fn render_gives_the_limit_and_never_the_bytes() {
        let mut tera = tera::Tera::default();
        tera.add_raw_template(
            "base_file.html",
            "{{ max_size_mb }}|{{ field.value }}|{{ is_file }}",
        )
        .unwrap();
        let mut field = BinaryField::new("blob").max_size(1_572_864);
        field.set_value("c2VjcmV0");
        assert_eq!(
            field.render(&std::sync::Arc::new(tera)).unwrap(),
            "1.5||true"
        );
    }
}

/// `binary`, `var_binary` and `blob` fields: an upload stored as bytes.
/// The whole chain is exercised: `parse_multipart` stages the upload, the
/// admin form takes it, `finalize` encodes it, the generated conversion
/// decodes it into the entity's `Vec<u8>`.
/// Moved from `tests/formulaire/test_binary_field.rs` when `parse_multipart`
/// became `pub(crate)`.
#[cfg(test)]
mod upload_tests {
    use crate::forms::Forms;
    use crate::forms::field::FormAccess;
    use crate::utils::forms::parse_html::parse_multipart;
    use axum::{
        Router,
        body::Body,
        extract::Multipart,
        http::{Method, Request, header},
        routing::post,
    };
    use tower::ServiceExt;

    // `model!` also writes `Doc::objects`, which these tests don't use.
    #[allow(dead_code)]
    mod docs {
        use runique::prelude::*;

        model! {
            Doc,
            table: "docs",
            pk: id => i32,
            {
                thumb: var_binary [nullable, max_length: 8],
                body: blob [nullable],
            }
        }
    }

    mod users_ext {
        use runique::prelude::*;

        extend! {
            table: "eihwaz_users",
            fields: {
                signature: blob [nullable],
            }
        }
    }

    /// Stages the request's uploads like Prisme, runs the `Doc` admin form, and
    /// returns the entity's bytes (`thumb`, `body`), or `invalid`.
    async fn handler(multipart: Multipart) -> String {
        let media = std::env::var("MEDIA_ROOT").expect("MEDIA_ROOT");
        let parsed = parse_multipart(multipart, std::path::Path::new(&media), 10, 64)
            .await
            .expect("multipart");
        let data = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
        let mut form = Forms::new("csrf");
        docs::DocModelForm::register_fields(&mut form);
        form.fill(&data, Method::POST);
        if !form.is_valid().await.unwrap_or(false) {
            return "invalid".into();
        }
        form.finalize().await.expect("finalize");
        let values = form
            .fields
            .iter()
            .map(|(k, f)| (k.clone(), f.value().to_string()))
            .collect();
        let model = match docs::admin_from_form(&values, None) {
            Ok(model) => model,
            Err(e) => return format!("error: {e}"),
        };
        let text = |value: runique::sea_orm::ActiveValue<Option<Vec<u8>>>| match value {
            runique::sea_orm::ActiveValue::Set(bytes) => {
                String::from_utf8_lossy(&bytes.unwrap_or_default()).into_owned()
            }
            _ => "unset".to_string(),
        };
        format!("{}|{}", text(model.thumb.clone()), text(model.body.clone()))
    }

    fn request(parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
        let boundary = "----rqbin";
        let mut body = String::new();
        for (name, filename, content) in parts {
            let disposition = match filename {
                Some(f) => format!("form-data; name=\"{name}\"; filename=\"{f}\""),
                None => format!("form-data; name=\"{name}\""),
            };
            body.push_str(&format!(
                "--{boundary}\r\nContent-Disposition: {disposition}\r\n\r\n{content}\r\n"
            ));
        }
        body.push_str(&format!("--{boundary}--\r\n"));
        Request::builder()
            .method("POST")
            .uri("/upload")
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn send(req: Request<Body>) -> String {
        let app = Router::new().route("/upload", post(handler));
        let resp = app.oneshot(req).await.expect("response");
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("body");
        String::from_utf8(bytes.to_vec()).expect("utf-8")
    }

    fn with_media<T>(run: impl FnOnce(&std::path::Path) -> T) -> T {
        let media = std::env::temp_dir().join(format!("rq_bin_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&media).unwrap();
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        let out = run(&media);
        crate::config::static_files::restore_media_root();
        out
    }

    #[tokio::test]
    async fn uploaded_bytes_reach_the_entity() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = with_media(|m| m.to_path_buf());
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        let out = send(request(&[
            ("thumb", Some("t.bin"), "ABC"),
            ("body", Some("b.bin"), "hello blob"),
        ]))
        .await;
        crate::config::static_files::restore_media_root();
        assert_eq!(out, "ABC|hello blob");
        let leftovers: Vec<_> = walk(&media);
        assert!(
            leftovers.is_empty(),
            "staged uploads removed: {leftovers:?}"
        );
        let _ = std::fs::remove_dir_all(&media);
    }

    #[tokio::test]
    async fn an_upload_longer_than_the_column_is_refused() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = with_media(|m| m.to_path_buf());
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        let out = send(request(&[("thumb", Some("t.bin"), "123456789")])).await;
        crate::config::static_files::restore_media_root();
        assert_eq!(out, "invalid", "var_binary [max_length: 8] holds 8 bytes");
        let _ = std::fs::remove_dir_all(&media);
    }

    #[tokio::test]
    async fn a_path_sent_as_text_is_never_read() {
        let _g = crate::config::static_files::MEDIA_ENV_LOCK.lock().await;
        let media = with_media(|m| m.to_path_buf());
        set_env("MEDIA_ROOT", media.to_str().unwrap());
        let secret = std::env::temp_dir().join(format!("rq_secret_{}", uuid::Uuid::new_v4()));
        std::fs::write(&secret, b"TOP SECRET").unwrap();
        let out = send(request(&[("body", None, secret.to_str().unwrap())])).await;
        crate::config::static_files::restore_media_root();
        assert_eq!(out, "unset|unset", "nothing read, nothing written");
        assert!(secret.exists());
        let _ = std::fs::remove_file(&secret);
        let _ = std::fs::remove_dir_all(&media);
    }

    #[test]
    fn schema_and_extend_forms_get_a_binary_field() {
        let mut form = Forms::new("csrf");
        docs::schema().fill_form(&mut form, None, None);
        assert_eq!(form.fields["body"].field_type(), "binary");

        let mut form = Forms::new("csrf");
        users_ext::EihwazUsersModelForm::register_fields(&mut form);
        assert_eq!(form.fields["signature"].field_type(), "binary");
    }

    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk(&path));
            } else {
                files.push(path);
            }
        }
        files
    }

    fn set_env(key: &str, val: &str) {
        // SAFETY: the tests touching MEDIA_ROOT hold `MEDIA_ENV_LOCK`.
        unsafe { std::env::set_var(key, val) }
    }
}
