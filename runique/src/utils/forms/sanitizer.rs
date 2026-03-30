use crate::utils::constante::parse::{ALLOWED_ATTRS, ALLOWED_TAGS, RICH_CONTENT_FIELDS};
use ammonia::Builder;
use std::{collections::HashSet, sync::LazyLock};

/// Builder ammonia pré-configuré — initialisé une seule fois, réutilisé à chaque requête.
static RICH_BUILDER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let mut builder = Builder::new();
    builder.tags(ALLOWED_TAGS.clone());
    builder.tag_attributes(ALLOWED_ATTRS.clone());
    builder.url_schemes(HashSet::from(["http", "https", "mailto"]));
    if !ALLOWED_ATTRS
        .get("a")
        .is_some_and(|attrs| attrs.contains("rel"))
    {
        builder.link_rel(Some("noopener noreferrer"));
    }
    builder
});

/// =============================
/// STRICT MODE — TEXT ONLY
/// =============================
///
/// - Removes ALL HTML tags
/// - Neutralizes dangerous protocols
/// - Usable everywhere without concern
#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer_strict.md")]
#[must_use]
pub fn sanitize_strict(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let cleaned = Builder::new().tags(HashSet::new()).clean(input).to_string();

    cleaned
        .to_lowercase()
        .replace("javascript:", "")
        .replace("vbscript:", "")
        .replace("data:", "")
        .replace("file:", "")
        .trim()
        .to_string()
}

/// =============================
/// RICH MODE — CONTROLLED HTML
/// =============================
///
/// - Strict allow-list
/// - No JS possible
/// - No SVG / `MathML`
#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer_rich.md")]
pub fn sanitize_rich(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    RICH_BUILDER.clean(input).to_string().trim().to_string()
}

/// =============================
/// SINGLE ENTRY POINT
/// =============================
///
/// Automatically decides the sanitation mode
#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer.md")]
pub fn sanitize(field: &str, input: &str) -> String {
    if RICH_CONTENT_FIELDS.contains(&field) {
        sanitize_rich(input)
    } else {
        sanitize_strict(input)
    }
}

/// =============================
/// XSS TESTS (OWASP)
/// =============================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_blocks_all_xss() {
        let payloads = [
            "<script>alert(1)</script>",
            "<img src=x onerror=alert(1)>",
            "javascript:alert(1)",
            "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",
            "<svg/onload=alert(1)>",
            "<a href=\"javascript:alert(1)\">x</a>",
        ];

        for p in payloads {
            let out = sanitize_strict(p);
            assert!(!out.contains("script"));
            assert!(!out.contains("javascript"));
            assert!(!out.contains("onerror"));
            assert!(!out.contains("data:"));
        }
    }

    #[test]
    fn rich_allows_basic_html() {
        let html = "<p><strong>Hello</strong> <a href=\"https://example.com\">link</a></p>";
        let out = sanitize_rich(html);

        assert!(out.contains("<p>"));
        assert!(out.contains("<strong>"));
        assert!(out.contains("href=\"https://example.com\""));
    }

    #[test]
    fn rich_blocks_script_and_events() {
        let html = "<p onclick=alert(1)>Test</p><script>alert(1)</script>";
        let out = sanitize_rich(html);

        assert!(!out.contains("script"));
        assert!(!out.contains("onclick"));
    }
}
