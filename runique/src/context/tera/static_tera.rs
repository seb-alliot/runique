//! Global registration of Tera filters/functions — `register_asset_filters` and `| markdown` filter.
use crate::context::tera::form::FormFilter;
use crate::context::tera::url::LinkFunction;
use crate::middleware::CsrfTokenFunction;
use crate::utils::aliases::ARlockmap;
use chrono::NaiveDateTime;
use pulldown_cmark::{Options, Parser, html};
use tera::{Filter, Kwargs, State, Tera, TeraResult, Value, escape_html};

// Filter to mask a sensitive value with bullets (real number of characters)
fn mask_filter(value: &str, _: Kwargs, _: &State) -> String {
    "•".repeat(value.chars().count())
}

// Filter to generate a hidden CSRF field.
//
// Implemented on a unit struct rather than as a plain function: only a `Filter`
// impl can override `is_safe()`, and that declaration is what replaces the `| safe`
// the preprocessor used to inject into the template source. The rule for every
// filter below is the same — sanitize (or build) the HTML first, mark it safe last.
pub struct CsrfFieldFilter;

impl Filter<&str, TeraResult<Value>> for CsrfFieldFilter {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, token: &str, _: Kwargs, _: &State) -> TeraResult<Value> {
        // The token comes from Runique's own generator, so its alphabet is already
        // attribute-safe. It is escaped anyway: the surrounding string is emitted
        // unescaped, so the safety of this tag must not rest on a guarantee made
        // in another module.
        let mut escaped = Vec::new();
        escape_html(token, &mut escaped)
            .map_err(|e| tera::Error::chain("csrf_field: failed to escape the token", e))?;
        let token = String::from_utf8(escaped)?;

        Ok(Value::safe_string(&format!(
            r#"<input type="hidden" name="csrf_token" value="{token}">"#
        )))
    }
}

// Filter to format a NaiveDateTime string → "dd/mm/yyyy HH:MM"
fn format_date_filter(value: &str, _: Kwargs, _: &State) -> String {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|_| value.to_string())
}

// Markdown filter → HTML (tables, strikethrough, heading ids)
pub struct MarkdownFilter;

impl Filter<&str, Value> for MarkdownFilter {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, md: &str, _: Kwargs, _: &State) -> Value {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        let parser = Parser::new_ext(md, opts);
        let mut output = String::new();
        html::push_html(&mut output, parser);
        // Emitted unescaped, so XSS in user-authored Markdown (raw <script>,
        // javascript: links…) is neutralized here. `safe_string` is applied to the
        // sanitized result only — never to `output` before this line.
        Value::safe_string(&crate::utils::sanitizer::sanitize_markdown(&output))
    }
}

// Re-sanitizes stored rich HTML at render time. The output is ammonia's own
// (XSS-free by construction), re-cleaned here regardless of how the value reached
// storage — sanitization happens on output, storage is never trusted.
pub struct SanitizeFilter;

impl Filter<&str, Value> for SanitizeFilter {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, raw: &str, _: Kwargs, _: &State) -> Value {
        Value::safe_string(&crate::utils::sanitizer::sanitize_rich(raw))
    }
}

// Plain-text projection of a (possibly rich) value: strips every tag and decodes
// entities, so a stored `&gt;` becomes a real `>` that Tera then escapes once.
// Used for list-cell previews where rendered block HTML would break the row.
//
// Deliberately a plain function, not a `Filter` impl: the blanket impl leaves
// `is_safe()` at `false`, and returning a `String` builds a normal (escapable)
// value. Both properties are load-bearing — this filter must stay auto-escaped,
// otherwise it would emit as HTML the very markup it was asked to strip.
fn plaintext_filter(raw: &str, _: Kwargs, _: &State) -> String {
    crate::utils::sanitizer::sanitize_strict(raw)
}

// Humanizes a machine identifier for display: splits on `_`/`-` and capitalizes
// each word ("changelog_entry" -> "Changelog Entry"). Output stays plain text
// (auto-escaped by Tera). Opt-in per template — apply to identifiers (enum
// values, resource keys, column names), never to raw user data (a username like
// `jean_dupont` would be silently altered).
fn humanize_filter(value: &Value, _: Kwargs, _: &State) -> Value {
    // Non-strings (numbers, bools) pass through untouched — applied on a generic
    // value loop, humanizing a number must never blank it out.
    let Some(raw) = value.as_str() else {
        return value.clone();
    };
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
    // Plain text: `From<String>` builds a normal (escapable) string, never a safe one.
    Value::from(humanized)
}

// Internal generic function to avoid repetition.
// A non-string input no longer needs a hand-written error: Tera rejects it on the
// `&str` argument and reports the offending value itself.
fn register_filter(base_url: String, version: String) -> impl Fn(&str, Kwargs, &State) -> String {
    move |file: &str, _: Kwargs, _: &State| {
        let full_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            file.trim_start_matches('/')
        );

        if version.is_empty() {
            full_url
        } else {
            format!(r#"{}?v={}"#, full_url, version)
        }
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

    // Filtres hérités de la feature `builtins` de Tera 1 (urlencode, slug, date…),
    // déplacés dans `tera-contrib` en 2.0. Enregistrés d'abord : ils font partie du
    // socle sur lequel les templates internes comptent.
    crate::context::tera::contrib::register_contrib(tera);

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
    // Filters declaring `is_safe()` — their output is HTML that Runique built and
    // sanitized, and it is emitted unescaped without any `| safe` in the template.
    tera.register_filter("form", FormFilter);
    tera.register_filter("csrf_field", CsrfFieldFilter);
    tera.register_filter("markdown", MarkdownFilter);
    tera.register_filter("sanitize", SanitizeFilter);
    // Auto-escaped like any plain value.
    tera.register_filter("plaintext", plaintext_filter);
    tera.register_filter("format_date", format_date_filter);
    tera.register_filter("humanize", humanize_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("link", LinkFunction { url_registry });
}
