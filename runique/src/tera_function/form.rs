use std::collections::HashMap;
use tera::Value;

pub fn form_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    // Si on demande un champ précis
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {
        if let Some(fields) = value.get("fields").and_then(|f| f.as_object()) {
            if let Some(field_html) = fields.get(field_name) {
                return Ok(field_html.clone());
            }
        }
        return Ok(Value::String(field_name.to_string()));
    }

    // Sinon, on rend tout le formulaire
    if let Some(html) = value.get("html") {
        return Ok(html.clone());
    }

    Ok(Value::Null)
}
