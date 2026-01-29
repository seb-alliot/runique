// Dans src/tera_function/form_filter.rs
use crate::aliases::JsonMap;
use crate::aliases::TResult;
use tera::Value;

pub fn form_filter(value: &Value, args: &JsonMap) -> TResult {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        // Essaie d'abord dans "fields"
        if let Some(fields) = value.get("fields").and_then(|f| f.as_object()) {
            if let Some(field_obj) = fields.get(field_name) {
                return Ok(field_obj.clone());
            }
        }

        // Essaie dans "form.fields"
        if let Some(form) = value.get("form") {
            if let Some(fields) = form.get("fields").and_then(|f| f.as_object()) {
                if let Some(field_obj) = fields.get(field_name) {
                    return Ok(field_obj.clone());
                }
            }

            // Essaie dans "form.cleaned_data"
            if let Some(cleaned_data) = form.get("cleaned_data").and_then(|f| f.as_object()) {
                if let Some(field_value) = cleaned_data.get(field_name) {
                    // Retourne la valeur du champ depuis cleaned_data
                    return Ok(field_value.clone());
                }
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
