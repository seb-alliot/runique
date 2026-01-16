// Dans src/tera_function/form_filter.rs
use std::collections::HashMap;
use tera::Value;

pub fn form_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        if let Some(fields) = value.get("fields").and_then(|f| f.as_object()) {
            if let Some(field_obj) = fields.get(field_name) {
                return Ok(field_obj.clone());
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
