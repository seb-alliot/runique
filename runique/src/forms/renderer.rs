//! HTML rendering of form fields via Tera with fallback to internal templates.
use crate::forms::base::FormField;
use crate::utils::{
    aliases::{ATera, FieldsMap},
    trad::tf,
};
use tracing::{debug, trace, warn};

#[derive(Clone)]
pub struct FormRenderer {
    tera: ATera,
    pub js_files: Vec<String>,
}

impl FormRenderer {
    pub fn new(tera: ATera) -> Self {
        Self {
            tera,
            js_files: Vec::new(),
        }
    }

    pub fn add_js(&mut self, files: &[&str]) {
        debug!(files_count = files.len(), "add JS files to form");

        for file in files {
            if let Some(reason) = Self::validate_js_path(file) {
                warn!(file = %file, reason = reason, "Skipping JS file");
                continue;
            }
            self.js_files.push(file.to_string());
            trace!(file = %file, "Added JS file to form");
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
                Ok(rendered) => html.push(rendered),
                Err(e) => return Err(tf("forms.finalize_error", &[field.name(), &e]).to_owned()),
            }
        }

        Ok(html.join("\n"))
    }

    fn render_js(&self) -> Result<String, String> {
        if self.js_files.is_empty() {
            return Ok(String::new());
        }

        let template_name = "js_files";
        if !self
            .tera
            .get_template_names()
            .any(|name| name == template_name)
        {
            return Err(tf("forms.template_missing", &[template_name]).to_owned());
        }

        let mut context = tera::Context::new();
        context.insert("js_files", &self.js_files);

        self.tera
            .render(template_name, &context)
            .map_err(|e| tf("forms.render_js_error", &[&e]).to_owned())
    }

    pub fn render_field(&self, field: &dyn FormField) -> Result<String, String> {
        field.render(&self.tera)
    }
}
