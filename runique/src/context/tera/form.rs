//! Tera `form` filter — HTML rendering of a form field by name from context.
// Dans src/tera_function/form_filter.rs
use crate::middleware::errors::error::html_escape;
use crate::utils::aliases::JsonMap;
use crate::utils::aliases::TResult;
use tera::Value;

pub fn form_filter(value: &Value, args: &JsonMap) -> TResult {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        let field_html = render_field(value, field_name)?;
        let mut output = field_html.as_str().unwrap_or("").to_string();

        // Inject CSRF before the first field
        if is_first_field_by_index(value, field_name)
            && let Some(csrf) = render_csrf(value)
        {
            output = format!("{}\n{}", csrf, output);
        }

        // Inject scripts after the last field
        if is_last_field_by_index(value, field_name) {
            let scripts = render_scripts(value).unwrap_or_default();
            if !scripts.is_empty() {
                output = format!(
                    "{}\n<!-- Form scripts (auto-injected after last field) -->\n{}",
                    output, scripts
                );
            }
        }

        return Ok(Value::String(output));
    }

    // Render full form
    render_form_html(value)
}

/// Checks if the field is the first by index (excluding csrf_token)
fn is_first_field_by_index(value: &Value, field_name: &str) -> bool {
    let fields = value
        .get("fields")
        .or_else(|| value.get("form").and_then(|f| f.get("fields")));

    let fields_obj = match fields.and_then(|f| f.as_object()) {
        Some(obj) => obj,
        None => return false,
    };

    // Minimum index excluding csrf_token
    let min_index = fields_obj
        .values()
        .filter(|f| f.get("name").and_then(|n| n.as_str()) != Some("csrf_token"))
        .filter_map(|field| field.get("index"))
        .filter_map(|idx| idx.as_u64())
        .min();

    let current_index = fields_obj
        .get(field_name)
        .and_then(|field| field.get("index"))
        .and_then(|idx| idx.as_u64());

    match (min_index, current_index) {
        (Some(min), Some(current)) => min == current,
        _ => false,
    }
}

/// Checks if the field is the last by index
fn is_last_field_by_index(value: &Value, field_name: &str) -> bool {
    let fields = value
        .get("fields")
        .or_else(|| value.get("form").and_then(|f| f.get("fields")));

    let fields_obj = match fields.and_then(|f| f.as_object()) {
        Some(obj) => obj,
        None => return false,
    };

    // Find maximum index (last field)
    let max_index = fields_obj
        .values()
        .filter_map(|field| field.get("index"))
        .filter_map(|idx| idx.as_u64())
        .max();

    // Get current field index
    let current_index = fields_obj
        .get(field_name)
        .and_then(|field| field.get("index"))
        .and_then(|idx| idx.as_u64());

    match (max_index, current_index) {
        (Some(max), Some(current)) => max == current,
        _ => false,
    }
}

/// Retrieves the CSRF field HTML from rendered_fields
fn render_csrf(value: &Value) -> Option<String> {
    let rendered = find_rendered_fields(value)?;
    rendered
        .get("csrf_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn render_scripts(value: &Value) -> Option<String> {
    let js_files = value
        .get("js_files")
        .or_else(|| value.get("form").and_then(|f| f.get("js_files")))?;

    let files = js_files.as_array()?;

    if files.is_empty() {
        return None;
    }

    let scripts: Vec<String> = files
        .iter()
        .filter_map(|f| f.as_str())
        .map(|file| {
            format!(
                r#"<script src="/static/{}" defer></script>"#,
                html_escape(file)
            )
        })
        .collect();

    if scripts.is_empty() {
        None
    } else {
        Some(scripts.join("\n"))
    }
}

fn render_field(value: &Value, field_name: &str) -> TResult {
    let rendered_fields = find_rendered_fields(value);

    if let Some(fields) = rendered_fields
        && let Some(html) = fields.get(field_name).and_then(|v| v.as_str())
    {
        return Ok(Value::String(html.to_string()));
    }

    Err(tera::Error::msg(format!(
        "Field '{}' not found in form",
        field_name
    )))
}

fn find_rendered_fields(value: &Value) -> Option<&Value> {
    value
        .get("rendered_fields")
        .or_else(|| value.get("form").and_then(|f| f.get("rendered_fields")))
}

/// Renders the complete form HTML
fn render_form_html(value: &Value) -> TResult {
    let html = find_html(value);

    if let Some(html_str) = html {
        return Ok(Value::String(html_str.to_string()));
    }

    Err(tera::Error::msg(
        "Cannot render form: no 'html' field found",
    ))
}

/// Intelligently searches for the "html" field
fn find_html(value: &Value) -> Option<String> {
    // 1. Directly value.html
    if let Some(html) = value.get("html").and_then(|v| v.as_str()) {
        return Some(html.to_string());
    }

    // 2. value.form.html (struct with "form" field)
    if let Some(form) = value.get("form")
        && let Some(html) = form.get("html").and_then(|v| v.as_str())
    {
        return Some(html.to_string());
    }

    // 3. Rebuild from rendered_fields if html is absent
    if let Some(fields) = find_rendered_fields(value)
        && let Some(obj) = fields.as_object()
    {
        let html: Vec<&str> = obj.values().filter_map(|v| v.as_str()).collect();
        if !html.is_empty() {
            return Some(html.join("\n"));
        }
    }

    None
}
