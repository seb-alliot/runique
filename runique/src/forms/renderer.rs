use crate::forms::base::FormField;
use crate::utils::aliases::{ATera, FieldsMap};
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

    pub fn render(&self, fields: &FieldsMap) -> Result<String, String> {
        let mut html = Vec::new();

        let js_html = self.render_js()?;
        if !js_html.is_empty() {
            html.push(js_html);
        }

        for field in fields.values() {
            match field.render(&self.tera) {
                Ok(rendered) => html.push(rendered),
                Err(e) => return Err(format!("Render error '{}': {}", field.name(), e)),
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
            return Err(format!("Missing template: {}", template_name));
        }

        let mut context = tera::Context::new();
        context.insert("js_files", &self.js_files);

        self.tera
            .render(template_name, &context)
            .map_err(|e| format!("JS render error: {}", e))
    }

    pub fn render_field(&self, field: &dyn FormField) -> Result<String, String> {
        field.render(&self.tera)
    }
}
