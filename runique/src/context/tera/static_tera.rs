//! Global registration of Tera filters/functions — `register_asset_filters` and `| markdown` filter.
use crate::context::tera::form::FormFilter;
use crate::context::tera::url::LinkFunction;
use crate::middleware::CsrfTokenFunction;
use crate::utils::aliases::{ARlockmap, StrMap};
use chrono::{DateTime, NaiveDateTime, NaiveTime};
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
/// Tera filter rendering a hidden `<input type="hidden" name="csrf_token">` field
/// from a raw CSRF token, HTML-escaping the token and marking the output safe.
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

/// A date-time as the admin shows it, in every page.
pub(crate) fn display_datetime(value: &NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// The readable form of a date-time or time as a row's JSON gives it; `None`
/// for anything else, a plain date included, which reads well as it is.
fn display_time_text(value: &str) -> Option<String> {
    if let Ok(with_offset) = DateTime::parse_from_rfc3339(value) {
        return Some(display_datetime(&with_offset.naive_local()));
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(display_datetime(&datetime));
    }
    NaiveTime::parse_from_str(value, "%H:%M:%S%.f")
        .ok()
        .map(|time| time.format("%H:%M:%S").to_string())
}

// Any other value goes through untouched: a row's cell can be a number too.
fn display_time_filter(value: Value, _: Kwargs, _: &State) -> Value {
    match value.as_str().and_then(display_time_text) {
        Some(text) => Value::from(text),
        None => value,
    }
}

// Markdown filter → HTML (tables, strikethrough, heading ids)
/// Tera filter that renders Markdown to sanitized HTML, marked safe for direct output.
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
/// Tera filter that re-sanitizes stored rich-text HTML at render time via `sanitize_rich`.
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
/// The URL of `file` under `base_url`, with the cache-busting `version`
/// when there is one: what `{% static %}` and its siblings write.
pub(crate) fn asset_url(base_url: &str, file: &str, version: &str) -> String {
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

fn register_filter(base_url: String, version: String) -> impl Fn(&str, Kwargs, &State) -> String {
    move |file: &str, _: Kwargs, _: &State| asset_url(&base_url, file, &version)
}

fn integrity_filter(hashes: StrMap) -> impl Fn(&str, Kwargs, &State) -> String {
    move |file: &str, _: Kwargs, _: &State| hashes.get(file).cloned().unwrap_or_default()
}

/// Registers `static_integrity`: the SRI hash of a project's static file, empty
/// when it has none. For a path only known at render time, which `{% static %}`
/// can't hash as it does a literal one. Registering again replaces the hashes.
pub fn register_integrity_filter(tera: &mut Tera, hashes: StrMap) {
    tera.register_filter("static_integrity", integrity_filter(hashes));
}

/// Registers Runique's built-in Tera filters and functions on `tera`: contrib
/// builtins (urlencode, slug, date…), `static`/`media`/`runique_static`/`runique_media`
/// URL filters (versioned via a CSS cache-busting token), `form`, `csrf_field`,
/// `markdown`, `sanitize`, `plaintext`, `format_date`, `display_time`, `humanize`, and the
/// `csrf_token`/`link` functions.
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
    // Tera refuses a template naming an unknown filter: `js.html` needs this one
    // to load. The template loader replaces it once the files are hashed.
    register_integrity_filter(tera, StrMap::new());
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
    tera.register_filter("display_time", display_time_filter);
    tera.register_filter("humanize", humanize_filter);
    tera.register_function("csrf_token", CsrfTokenFunction);
    tera.register_function("link", LinkFunction { url_registry });
}

#[cfg(test)]
mod display_time_tests {
    use super::*;

    fn render(value: impl Into<Value>) -> String {
        let mut tera = Tera::default();
        tera.register_filter("display_time", display_time_filter);
        tera.add_raw_template("t", "{{ value | display_time }}")
            .unwrap();
        let mut context = tera::Context::new();
        context.insert("value", &value.into());
        tera.render("t", &context).unwrap()
    }

    #[test]
    fn date_times_read_without_t_or_fraction() {
        for value in [
            "2026-11-06T14:58:32.907849",
            "2026-11-06T14:58:32",
            "2026-11-06T14:58:32+00:00",
            "2026-11-06T14:58:32Z",
        ] {
            assert_eq!(render(value), "2026-11-06 14:58:32", "{value}");
        }
    }

    #[test]
    fn an_offset_keeps_the_stored_local_time() {
        assert_eq!(render("2026-11-06T14:58:32+02:00"), "2026-11-06 14:58:32");
    }

    #[test]
    fn a_time_loses_its_fraction() {
        assert_eq!(render("14:58:32.907849"), "14:58:32");
        assert_eq!(render("14:58:32"), "14:58:32");
    }

    #[test]
    fn anything_else_stays_as_it_is() {
        assert_eq!(render("2026-11-06"), "2026-11-06");
        assert_eq!(render("Corte, 14:58"), "Corte, 14:58");
        assert_eq!(render(42), "42");
    }

    #[test]
    fn a_plain_value_is_still_escaped() {
        assert_eq!(render("<b>"), "&lt;b&gt;");
    }

    #[test]
    fn the_shared_format_has_no_fraction() {
        let value =
            NaiveDateTime::parse_from_str("2026-11-06T14:58:32.5", "%Y-%m-%dT%H:%M:%S%.f").unwrap();
        assert_eq!(display_datetime(&value), "2026-11-06 14:58:32");
    }
}
