// Dans src/tera_function/form_filter.rs
use crate::utils::aliases::JsonMap;
use crate::utils::aliases::TResult;
use tera::Value;

pub fn form_filter(value: &Value, args: &JsonMap) -> TResult {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        // Cherche d'abord dans rendered_fields (HTML déjà rendu par Tera en Rust)
        if let Some(rendered) = value
            .get("rendered_fields")
            .and_then(|rf| rf.get(field_name))
            .and_then(|v| v.as_str())
        {
            return Ok(Value::String(rendered.to_string()));
        }

        // Même chose via form.rendered_fields
        if let Some(form) = value.get("form") {
            if let Some(rendered) = form
                .get("rendered_fields")
                .and_then(|rf| rf.get(field_name))
                .and_then(|v| v.as_str())
            {
                return Ok(Value::String(rendered.to_string()));
            }
        }

        return Err(tera::Error::msg(format!(
            "Field '{}' not found in form",
            field_name
        )));
    }

    if let Some(html) = value.get("html") {
        return Ok(html.clone());
    }

    Ok(Value::Null)
}
