//! Global registration of Tera filters/functions — `register_asset_filters` and `| markdown` filter.
use crate::context::tera::form::form_filter;
use crate::context::tera::url::LinkFunction;
use crate::middleware::CsrfTokenFunction;
use crate::utils::aliases::{ARlockmap, JsonMap, TResult};
use crate::utils::trad::tf;
use chrono::NaiveDateTime;
use pulldown_cmark::{Options, Parser, html};
use tera::{Tera, Value};

// Filter to mask a sensitive value with bullets (real number of characters)
fn mask_filter(value: &Value, _: &JsonMap) -> TResult {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String("•".repeat(s.chars().count())))
}

// Filter to generate a hidden CSRF field
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

// Filter to format a NaiveDateTime string → "dd/mm/yyyy HH:MM"
fn format_date_filter(value: &Value, _: &JsonMap) -> TResult {
    let s = value.as_str().unwrap_or("");
    let formatted = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|_| s.to_string());
    Ok(Value::String(formatted))
}

// Markdown filter → HTML (tables, strikethrough, heading ids)
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

// Internal generic function to avoid repetition
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
            format!(r#"{}?v={}"#, full_url, version)
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
    tera.register_filter("format_date", format_date_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("link", LinkFunction { url_registry });
}
