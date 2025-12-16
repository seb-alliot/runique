// rusti/src/tera_function/static_balise.rs
use tera::Tera;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub fn register_static_function(tera: &mut Tera, static_url: String) {
    // Cloner static_url pour l'utiliser dans la closure
    let static_file = static_url.clone();

    tera.register_function("static", move |args: &HashMap<String, JsonValue>| -> tera::Result<JsonValue> {
        // Récupérer l'argument 'file'
        let file = args
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("L'argument 'file' est requis et doit être une chaîne"))?;

        println!("Static function called with file: {}", file);

        // Construire l'URL complète
        let full_url = format!(
            "{}/{}",
            static_file.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        println!("Generated URL: {}", full_url);

        Ok(JsonValue::String(full_url))
    });
}

pub fn register_media_function(tera: &mut Tera, media_url: String) {
    let media_file: String = media_url.clone();

    tera.register_function("media", move |args: &HashMap<String, JsonValue>| -> tera::Result<JsonValue> {
        let file = args
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("L'argument 'file' est requis et doit être une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            media_file.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(JsonValue::String(full_url))
    });
}

// rusti/src/tera_function/static_balise.rs

// ... tes fonctions existantes ...

pub fn register_rusti_static(tera: &mut Tera, static_rusti_url: String) {
    let static_rusti_url_clone = static_rusti_url.clone();

    tera.register_function("rusti_static", move |args: &HashMap<String, JsonValue>| -> tera::Result<JsonValue> {
        let file = args
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("L'argument 'file' est requis et doit être une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            static_rusti_url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(JsonValue::String(full_url))
    });
}

pub fn register_rusti_media(tera: &mut Tera, media_rusti_url: String) {
    let media_rusti_url_clone = media_rusti_url.clone();

    tera.register_function("rusti_media", move |args: &HashMap<String, JsonValue>| -> tera::Result<JsonValue> {
        let file = args
            .get("file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("L'argument 'file' est requis et doit être une chaîne"))?;

        let full_url = format!(
            "{}/{}",
            media_rusti_url_clone.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(JsonValue::String(full_url))
    });
}