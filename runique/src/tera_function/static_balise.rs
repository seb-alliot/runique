use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::{Result as TeraResult, Tera, Value};

// Fonction générique interne pour éviter la répétition
fn generic_url_filter(
    base_url: String,
) -> impl Fn(&Value, &HashMap<String, Value>) -> TeraResult<Value> {
    move |value: &Value, _: &HashMap<String, Value>| {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg(format!("Erreur Runique : Le filtre static/media a reçu une valeur invalide ({:?}) au lieu d'un chemin de fichier.", value)))?;

        let full_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    }
}

pub fn register_all_asset_filters(
    tera: &mut Tera,
    static_url: String,
    media_url: String,
    runique_static_url: String,
    runique_media_url: String,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
) {
    tera.register_filter("static", generic_url_filter(static_url));
    tera.register_filter("media", generic_url_filter(media_url));
    tera.register_filter("runique_static", generic_url_filter(runique_static_url));
    tera.register_filter("runique_media", generic_url_filter(runique_media_url));
    tera.register_filter("form", crate::tera_function::form::form_filter);

    crate::tera_function::url_balise::register_link_function(tera, url_registry);
}
