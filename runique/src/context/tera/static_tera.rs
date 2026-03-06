use crate::context::tera::csp::nonce_function;
use crate::utils::trad::tf;
use crate::context::tera::form::form_filter;
use crate::context::tera::url::LinkFunction;
use crate::middleware::CsrfTokenFunction;
use crate::utils::aliases::{ARlockmap, JsonMap, TResult};
use tera::{Tera, Value};

// Filtre pour masquer une valeur sensible avec des bullets (nombre de caractères réel)
fn mask_filter(value: &Value, _: &JsonMap) -> TResult {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String("•".repeat(s.chars().count())))
}

// Filtre pour générer un champ CSRF hidden
fn csrf_filter(value: &Value, _: &JsonMap) -> TResult {
    let token = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("csrf_field filter requires a string token value"))?;

    let html = format!(
        r#"<input type="hidden" name="csrf_token" value="{}">"#,
        token
    );

    Ok(Value::String(html))
}

// Fonction générique interne pour éviter la répétition
fn register_filter(base_url: String) -> impl Fn(&Value, &JsonMap) -> TResult {
    move |value: &Value, _: &JsonMap| {
        let file = value.as_str()
            .ok_or_else(|| tera::Error::msg(tf("forms.filter_invalid_value", &[&format!("{:?}", value)])))?;

        let full_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        Ok(Value::String(full_url))
    }
}

pub fn register_asset_filters(
    tera: &mut Tera,
    static_url: String,
    media_url: String,
    runique_static_url: String,
    runique_media_url: String,
    url_registry: ARlockmap,
) {
    tera.register_filter("mask", mask_filter);
    tera.register_filter("static", register_filter(static_url));
    tera.register_filter("media", register_filter(media_url));
    tera.register_filter("runique_static", register_filter(runique_static_url));
    tera.register_filter("runique_media", register_filter(runique_media_url));
    tera.register_filter("form", form_filter);
    tera.register_filter("csrf_field", csrf_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("nonce", nonce_function);
    tera.register_function("link", LinkFunction { url_registry });
}
