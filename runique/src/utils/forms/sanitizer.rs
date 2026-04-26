//! HTML sanitization via ammonia — pre-configured builder for rich-text, whitelist of tags/attributes.
/// Pre-configured ammonia builder — initialized once, reused for each request.
use crate::utils::constante::parse::{ALLOWED_ATTRS, ALLOWED_TAGS, RICH_CONTENT_FIELDS};
use ammonia::Builder;
use std::{collections::HashSet, sync::LazyLock};

/// Pre-configured ammonia builder — initialized once, reused for each request.
static RICH_BUILDER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let mut builder = Builder::new();

    // Allowed tags and attributes (whitelist)
    builder.tags(ALLOWED_TAGS.clone());
    builder.tag_attributes(ALLOWED_ATTRS.clone());

    // Strict URL schemes
    builder.url_schemes(HashSet::from(["http", "https"]));

    // Disallow relative URLs (open redirect prevention)
    builder.url_relative(ammonia::UrlRelative::Deny);

    // Force rel="noopener noreferrer" on all links
    builder.link_rel(Some("noopener noreferrer"));

    // Allow target="_blank" only (blocks _top, _parent, etc.)
    builder.add_tag_attributes("a", &["target"]);
    builder.add_tag_attribute_values("a", "target", &["_blank"]);

    // Strip HTML comments
    builder.strip_comments(true);

    builder
});

// =============================
// STRICT MODE — TEXT ONLY
// =============================

#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer_strict.md")]
#[must_use]
pub fn sanitize_strict(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Strip all HTML tags, keep text content (spaces preserved, case preserved)
    let stripped = Builder::new().tags(HashSet::new()).clean(input).to_string();

    // Remove dangerous protocols (case-insensitive)
    const PROTOCOLS: &[&str] = &["javascript:", "vbscript:", "data:", "file:"];
    let mut result = stripped;
    for proto in PROTOCOLS {
        let lower = result.to_lowercase();
        if !lower.contains(proto) {
            continue;
        }
        let mut new_result = String::with_capacity(result.len());
        let mut last_end = 0;
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find(proto) {
            let abs_pos = search_from + pos;
            new_result.push_str(&result[last_end..abs_pos]);
            last_end = abs_pos + proto.len();
            search_from = last_end;
        }
        new_result.push_str(&result[last_end..]);
        result = new_result;
    }

    result.trim().to_string()
}

// =============================
// RICH MODE — CONTROLLED HTML
// =============================

#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer_rich.md")]
#[must_use]
pub fn sanitize_rich(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    RICH_BUILDER.clean(input).to_string().trim().to_string()
}

// =============================
// SINGLE ENTRY POINT
// =============================

#[doc = include_str!("../../../doc-tests/sanitizer/sanitizer.md")]
#[must_use]
pub fn sanitize(field: &str, input: &str) -> String {
    if RICH_CONTENT_FIELDS.contains(field) {
        sanitize_rich(input)
    } else {
        sanitize_strict(input)
    }
}

// =============================
// ADDITIONAL VALIDATION
// =============================

/// Checks if the cleaned content still contains suspicious elements
#[must_use]
pub fn is_suspicious_content(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("<script")
        || lower.contains("javascript:")
        || lower.contains("onerror=")
        || lower.contains("onload=")
        || lower.contains("data:text/html")
}

/// Cleans with fallback if suspicious content is detected
#[must_use]
pub fn sanitize_with_fallback(field: &str, input: &str, fallback: &str) -> String {
    let cleaned = sanitize(field, input);
    if is_suspicious_content(&cleaned) {
        fallback.to_string()
    } else {
        cleaned
    }
}

/// =============================
/// XSS TESTS (OWASP + extras)
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
            "<iframe src=javascript:alert(1)>",
            "<object data=javascript:alert(1)>",
            "<embed src=javascript:alert(1)>",
            "<form action=javascript:alert(1)><button>click</button></form>",
            "<input onfocus=alert(1) autofocus>",
            "<marquee onstart=alert(1)>",
            "<details ontoggle=alert(1)>",
            "<math href=\"javascript:alert(1)\">CLICKME</math>",
            "<mml:math href=\"javascript:alert(1)\">CLICKME</mml:math>",
        ];

        for p in payloads {
            let out = sanitize_strict(p);
            assert!(!out.contains("<"), "Failed on: {}", p); // Must be completely escaped
        }
    }

    #[test]
    fn rich_allows_basic_html() {
        let html = "<p><strong>Hello</strong> <a href=\"https://example.com\">link</a></p>";
        let out = sanitize_rich(html);

        assert!(out.contains("<p>"));
        assert!(out.contains("<strong>"));
        assert!(out.contains("href=\"https://example.com\""));
        assert!(out.contains("rel=\"noopener noreferrer\"") || !out.contains("target="));
    }

    #[test]
    fn rich_blocks_script_and_events() {
        let html = "<p onclick=alert(1)>Test</p><script>alert(1)</script>";
        let out = sanitize_rich(html);

        assert!(!out.contains("script"));
        assert!(!out.contains("onclick"));
        assert!(out.contains("Test")); // Text is preserved
    }

    #[test]
    fn rich_blocks_svg_mathml() {
        let html = r#"<svg onload=alert(1)><circle r=10></circle></svg><math href="javascript:alert(1)">x</math>"#;
        let out = sanitize_rich(html);

        assert!(!out.contains("<svg"));
        assert!(!out.contains("<math"));
        assert!(!out.contains("onload"));
        assert!(!out.contains("javascript"));
    }

    #[test]
    fn rich_adds_rel_to_links() {
        let html = r#"<a href="https://example.com" target="_blank">link</a>"#;
        let out = sanitize_rich(html);

        assert!(out.contains("rel=\"noopener noreferrer\""));
    }

    #[test]
    fn rich_blocks_mailto_if_not_allowed() {
        // If you haven't put "mailto" in url_schemes
        let html = r#"<a href="mailto:test@example.com">email</a>"#;
        let out = sanitize_rich(html);

        // The href must be removed or cleaned
        assert!(!out.contains("mailto:"));
    }

    #[test]
    fn strict_handles_unicode_obfuscation() {
        let html = "<scr\u{0000}ipt>alert(1)</script>"; // Null byte injection
        let out = sanitize_strict(html);
        assert!(!out.contains("<")); // Eveything must be escaped, regardless of the word
    }

    #[test]
    fn sanitize_routing_works() {
        assert_eq!(sanitize("content", "<p>Test</p>"), "<p>Test</p>");
        assert_eq!(sanitize("username", "<p>Test</p>"), "Test");
    }
}
