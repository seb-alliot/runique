use std::collections::HashMap;
use tera::Value;

pub fn form_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    if let Some(field_name) = args.get("field").and_then(|v| v.as_str()) {

        // On cherche avec le nom BRUT (ex: "username")
        if let Some(fields) = value.get("fields").and_then(|f| f.as_object()) {
            if let Some(field_obj) = fields.get(field_name) {

                // C'est ICI qu'on crée l'ID pour le HTML
                let html_id = format!("id_{}", field_name);
                let label = field_obj.get("label").and_then(|l| l.as_str()).unwrap_or(field_name);

                let html = format!(
                    r#"<div class="mb-3">
                        <label for="{id}">{label}</label>
                        <input name="{name}" id="{id}" class="form-control">
                    </div>"#,
                    id = html_id,
                    label = label,
                    name = field_name
                );
                return Ok(Value::String(html));
            }
        }
    }

    // 2. Cas : On ne demande rien ({{ form | form }}), on rend tout le formulaire
    if let Some(html) = value.get("html") {
        return Ok(html.clone());
    }

    Ok(Value::Null)
}
