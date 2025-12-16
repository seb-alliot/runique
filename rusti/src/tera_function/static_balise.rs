// rusti/src/tera_function/static_balise.rs
use tera::Tera;
use std::collections::HashMap;
use serde_json::Value;

// Balise filtré html pour simplifier l'appel dans les templates
// Exemple d'utilisation: {{ "css/style.css" | static }}
pub fn register_static(tera: &mut Tera, static_url: String) {
    let static_url_clone = static_url.clone();

    tera.register_filter("static", move |value: &Value, _: &HashMap<String, Value>| -> tera::Result<Value> {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg("Le filtre static nécessite une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            static_url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    });
}

// Exemple d'utilisation: {{ "images/photo.jpg" | media }}
pub fn register_media(tera: &mut Tera, media_url: String) {
    let media_url_clone = media_url.clone();

    tera.register_filter("media", move |value: &Value, _: &HashMap<String, Value>| -> tera::Result<Value> {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg("Le filtre media nécessite une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            media_url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    });
}

pub fn rusti_static(tera: &mut Tera, static_rusti_url: String) {
    let url_clone = static_rusti_url.clone();

    tera.register_filter("rusti_static", move |value: &Value, _: &HashMap<String, Value>| -> tera::Result<Value> {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg("Le filtre rusti_static nécessite une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    });
}

pub fn rusti_media(tera: &mut Tera, media_rusti_url: String) {
    let url_clone = media_rusti_url.clone();

    tera.register_filter("rusti_media", move |value: &Value, _: &HashMap<String, Value>| -> tera::Result<Value> {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg("Le filtre rusti_media nécessite une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    });
}