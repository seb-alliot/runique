//! Regex for Tera tags in templates — `{% static %}`, `{% media %}`, `{% link %}`.
use regex::Regex;
use std::sync::LazyLock;

pub static BALISE_LINK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{%\s*(?P<tag>static|media)\s*"(?P<link>[^"]+)"\s*%}"#).unwrap()
});

/// Handles `{% media var %}` / `{% static var %}` with a Tera variable (no quotes).
pub static BALISE_LINK_VAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{%\s*(?P<tag>static|media)\s+(?P<var>[a-zA-Z_][a-zA-Z0-9_.\[\]]*)\s*%}"#)
        .unwrap()
});

pub static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s*(?:,\s*)?(?P<params>[^%]*?)\s*%}"#)
        .unwrap()
});

pub static FORM_FIELD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{%\s*form\.([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)\s*%}").unwrap());

pub static FORM_FULL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{%\s*form\.([a-zA-Z0-9_]+)\s*%}").unwrap());

pub static MARKDOWN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*([^|{}\n]+?)\s*\|\s*markdown\s*\}\}").unwrap());

/// Matches `{{ form_fields.html }}` — Runique-generated HTML, never user input.
/// Rewrites to `{{ form_fields.html | safe }}` during template preprocessing.
pub static ADMIN_FORM_HTML_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*form_fields\.html\s*\}\}").unwrap());
