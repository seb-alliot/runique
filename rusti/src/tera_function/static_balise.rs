use tera::{Tera, Value, Result as TeraResult};
use std::collections::HashMap;
use tera::{Filter, to_value};

// Fonction générique interne pour éviter la répétition
fn generic_url_filter(base_url: String) -> impl Fn(&Value, &HashMap<String, Value>) -> TeraResult<Value> {
    move |value: &Value, _: &HashMap<String, Value>| {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg(format!("Erreur Rusti : Le filtre static/media a reçu une valeur invalide ({:?}) au lieu d'un chemin de fichier.", value)))?;

        let full_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    }
}



pub struct CsrfTokenFilter;

impl Filter for CsrfTokenFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        // 'value' est ce qui se trouve à gauche du pipe | (donc ton token brut)
        let token = value.as_str().unwrap_or("");

        let html = format!(
            r#"<input type="hidden" name="csrf_token" value="{}">"#,
            token
        );

        Ok(to_value(html)?)
    }
}

pub fn register_all_asset_filters(tera: &mut Tera, static_url: String, media_url: String, rusti_static_url: String, rusti_media_url: String) {
    tera.register_filter("static", generic_url_filter(static_url));
    tera.register_filter("media", generic_url_filter(media_url));
    tera.register_filter("rusti_static", generic_url_filter(rusti_static_url));
    tera.register_filter("rusti_media", generic_url_filter(rusti_media_url));
    tera.register_filter("csrf_token", CsrfTokenFilter);
}