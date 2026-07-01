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
    // The template preprocessor forces `| safe` on every `| markdown`, so the
    // output is emitted unescaped. Sanitize here to neutralize XSS in
    // user-authored Markdown (raw <script>, javascript: links, etc.).
    let output = crate::utils::sanitizer::sanitize_markdown(&output);
    Ok(Value::String(output))
}

// Re-sanitizes stored rich HTML at render time. The preprocessor forces `| safe`
// on every `| sanitize`, so the output is emitted unescaped — but it is ammonia's
// own (XSS-free by construction) output, re-cleaned here regardless of how the
// value reached storage.
fn sanitize_filter(value: &Value, _: &JsonMap) -> TResult {
    let raw = value.as_str().unwrap_or("");
    Ok(Value::String(crate::utils::sanitizer::sanitize_rich(raw)))
}

// Plain-text projection of a (possibly rich) value: strips every tag and decodes
// entities, so a stored `&gt;` becomes a real `>` that Tera then escapes once.
// No `| safe` is forced — the output is plain text and must stay auto-escaped.
// Used for list-cell previews where rendered block HTML would break the row.
fn plaintext_filter(value: &Value, _: &JsonMap) -> TResult {
    let raw = value.as_str().unwrap_or("");
    Ok(Value::String(crate::utils::sanitizer::sanitize_strict(raw)))
}

// Humanizes a machine identifier for display: splits on `_`/`-` and capitalizes
// each word ("changelog_entry" -> "Changelog Entry"). Output stays plain text
// (auto-escaped by Tera). Opt-in per template — apply to identifiers (enum
// values, resource keys, column names), never to raw user data (a username like
// `jean_dupont` would be silently altered).
fn humanize_filter(value: &Value, _: &JsonMap) -> TResult {
    let raw = value.as_str().unwrap_or("");
    let humanized = raw
        .split(['_', '-'])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Value::String(humanized))
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
    tera.register_filter("sanitize", sanitize_filter);
    tera.register_filter("plaintext", plaintext_filter);
    tera.register_filter("format_date", format_date_filter);
    tera.register_filter("humanize", humanize_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("link", LinkFunction { url_registry });
}
