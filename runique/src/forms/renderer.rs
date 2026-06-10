//! HTML rendering of form fields via Tera with fallback to internal templates.
use crate::forms::base::FormField;
use crate::utils::{
    aliases::{ATera, FieldsMap},
    trad::tf,
};
use tracing::warn;

#[derive(Clone)]
pub struct FormRenderer {
    tera: ATera,
    pub js_files: Vec<String>,
    csp_nonce: Option<String>,
}

impl FormRenderer {
    pub fn new(tera: ATera) -> Self {
        Self {
            tera,
            js_files: Vec::new(),
            csp_nonce: None,
        }
    }

    pub fn set_nonce(&mut self, nonce: impl Into<String>) {
        self.csp_nonce = Some(nonce.into());
    }

    pub fn add_js(&mut self, files: &[&str]) {
        for file in files {
            if let Some(reason) = Self::validate_js_path(file) {
                warn!(file = %file, reason = reason, "Skipping JS file");
                continue;
            }
            self.js_files.push(file.to_string());
        }
    }

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
        None
    }

    pub fn render(&self, fields: &FieldsMap, errors: &[String]) -> Result<String, String> {
        let log_render = crate::utils::runique_log::get_log()
            .forms
            .as_ref()
            .and_then(|f| f.render);
        if let Some(level) = log_render {
            crate::runique_log!(
                level,
                fields = fields.len(),
                global_errors = errors.len(),
                "render start"
            );
            if crate::utils::env::is_debug() {
                eprintln!(
                    "[runique:form] render start  fields={} global_errors={}",
                    fields.len(),
                    errors.len()
                );
            }
        }
        let mut html = Vec::new();

        // Render global errors first
        if !errors.is_empty() {
            let mut _context = tera::Context::new();
            _context.insert("errors", errors);
            html.push(
                errors
                    .iter()
                    .map(|err| format!("<div class=\"form-error\">{}</div>", err))
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }

        let js_html = self.render_js()?;
        if !js_html.is_empty() {
            html.push(js_html);
        }

        for field in fields.values() {
            match field.render(&self.tera) {
                Ok(rendered) => {
                    if let Some(level) = log_render {
                        crate::runique_log!(level, field = %field.name(), "rendered ok");
                        if crate::utils::env::is_debug() {
                            eprintln!("[runique:form] render ok  {}", field.name());
                        }
                    }
                    html.push(rendered);
                }
                Err(e) => {
                    if let Some(level) = log_render {
                        crate::runique_log!(level, field = %field.name(), error = %e, "render error");
                        if crate::utils::env::is_debug() {
                            eprintln!("[runique:form] render error  {} : {}", field.name(), e);
                        }
                    }
                    return Err(tf("forms.finalize_error", &[field.name(), &e]).to_owned());
                }
            }
        }

        Ok(html.join("\n"))
    }

    fn render_js(&self) -> Result<String, String> {
        if self.js_files.is_empty() {
            return Ok(String::new());
        }

        let template_name = "js_files.html";
        if !self
            .tera
            .get_template_names()
            .any(|name| name == template_name)
        {
            return Err(tf("forms.template_missing", &[template_name]).to_owned());
        }

        let mut context = tera::Context::new();
        context.insert("js_files", &self.js_files);
        if let Some(ref nonce) = self.csp_nonce {
            context.insert("csp_nonce", nonce);
        }

        self.tera
            .render(template_name, &context)
            .map_err(|e| tf("forms.render_js_error", &[&e]).to_owned())
    }

    pub fn render_field(&self, field: &dyn FormField) -> Result<String, String> {
        field.render(&self.tera)
    }
}
