// Dans src/tera_function/form_filter.rs
use crate::utils::aliases::JsonMap;
use crate::utils::aliases::TResult;
use tera::Value;

// Dans src/tera_function/form_filter.rs

pub fn form_filter(value: &Value, args: &JsonMap) -> TResult {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        // Rendre le field
        let field_html = render_field(value, field_name)?;

        // Vérifier si c'est le dernier field (par index)
        if is_last_field_by_index(value, field_name) {
            let scripts = render_scripts(value).unwrap_or_default();

            if !scripts.is_empty() {
                let combined = format!(
                    "{}\n<!-- Form scripts (auto-injected after last field) -->\n{}",
                    field_html.as_str().unwrap_or(""),
                    scripts
                );
                return Ok(Value::String(combined));
            }
        }

        return Ok(field_html);
    }

    // Rendu du form complet
    render_form_html(value)
}

/// Vérifie si le field est le dernier par index
fn is_last_field_by_index(value: &Value, field_name: &str) -> bool {
    // Chercher fields dans value ou value.form
    let fields = value
        .get("fields")
        .or_else(|| value.get("form").and_then(|f| f.get("fields")));

    let fields_obj = match fields.and_then(|f| f.as_object()) {
        Some(obj) => obj,
        None => return false,
    };

    // Trouver l'index maximum (dernier field)
    let max_index = fields_obj
        .values()
        .filter_map(|field| field.get("index"))
        .filter_map(|idx| idx.as_u64())
        .max();

    // Obtenir l'index du field actuel
    let current_index = fields_obj
        .get(field_name)
        .and_then(|field| field.get("index"))
        .and_then(|idx| idx.as_u64());

    // Comparer
    match (max_index, current_index) {
        (Some(max), Some(current)) => max == current,
        _ => false,
    }
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
        .map(|file| format!(r#"<script src="/static/{}" defer></script>"#, file))
        .collect();

    if scripts.is_empty() {
        None
    } else {
        Some(scripts.join("\n"))
    }
}

fn render_field(value: &Value, field_name: &str) -> TResult {
    let rendered_fields = find_rendered_fields(value);

    if let Some(fields) = rendered_fields {
        if let Some(html) = fields.get(field_name).and_then(|v| v.as_str()) {
            return Ok(Value::String(html.to_string()));
        }
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

/// Rendu du HTML complet du formulaire
fn render_form_html(value: &Value) -> TResult {
    // Chercher html à plusieurs endroits
    let html = find_html(value);

    if let Some(html_str) = html {
        return Ok(Value::String(html_str.to_string()));
    }

    Err(tera::Error::msg(
        "Cannot render form: no 'html' field found",
    ))
}

/// Cherche le champ "html" intelligemment
fn find_html(value: &Value) -> Option<String> {
    // 1. Directement value.html
    if let Some(html) = value.get("html").and_then(|v| v.as_str()) {
        return Some(html.to_string());
    }

    // 2. value.form.html (struct avec champ "form")
    if let Some(form) = value.get("form") {
        if let Some(html) = form.get("html").and_then(|v| v.as_str()) {
            return Some(html.to_string());
        }
    }

    // 3. Reconstruire depuis rendered_fields si html absent
    if let Some(fields) = find_rendered_fields(value) {
        if let Some(obj) = fields.as_object() {
            let html: Vec<&str> = obj.values().filter_map(|v| v.as_str()).collect();

            if !html.is_empty() {
                return Some(html.join("\n"));
            }
        }
    }

    None
}
