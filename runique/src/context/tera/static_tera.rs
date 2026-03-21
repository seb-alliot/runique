use crate::context::tera::csp::nonce_function;
use crate::context::tera::form::form_filter;
use crate::context::tera::url::LinkFunction;
use crate::middleware::CsrfTokenFunction;
use crate::utils::aliases::{ARlockmap, JsonMap, TResult};
use crate::utils::trad::tf;
use pulldown_cmark::{Options, Parser, html};
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

// Filtre markdown → HTML (tables, strikethrough, heading ids)
fn markdown_filter(value: &Value, _: &JsonMap) -> TResult {
    let md = value.as_str().unwrap_or("");
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(md, opts);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    Ok(Value::String(output))
}

// Fonction générique interne pour éviter la répétition
fn register_filter(base_url: String, version: String) -> impl Fn(&Value, &JsonMap) -> TResult {
    move |value: &Value, _: &JsonMap| {
        let file = value.as_str().ok_or_else(|| {
            tera::Error::msg(tf("forms.filter_invalid_value", &[&format!("{:?}", value)]))
        })?;

        let full_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        let url = if version.is_empty() {
            full_url
        } else {
            format!("{}?v={}", full_url, version)
        };

        Ok(Value::String(url))
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
    let version = crate::utils::env::css_token();
    tera.register_filter("mask", mask_filter);
    tera.register_filter("static", register_filter(static_url, version.clone()));
    tera.register_filter("media", register_filter(media_url, String::new()));
    tera.register_filter(
        "runique_static",
        register_filter(runique_static_url, version),
    );
    tera.register_filter(
        "runique_media",
        register_filter(runique_media_url, String::new()),
    );
    tera.register_filter("form", form_filter);
    tera.register_filter("csrf_field", csrf_filter);
    tera.register_filter("markdown", markdown_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("nonce", nonce_function);
    tera.register_function("link", LinkFunction { url_registry });
}
