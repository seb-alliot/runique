use crate::utils::constante::parse::{ALLOWED_ATTRS, ALLOWED_TAGS, RICH_CONTENT_FIELDS};
use ammonia::Builder;
use std::collections::HashSet;

/// =============================
/// STRICT MODE — TEXT ONLY
/// =============================
///
/// - Removes ALL HTML tags
/// - Neutralizes dangerous protocols
/// - Usable everywhere without concern
///
/// # Exemple
///
/// ```rust
/// use runique::utils::forms::sanitizer::sanitize_strict;
/// assert_eq!(sanitize_strict("<b>test</b>"), "test");
/// assert_eq!(sanitize_strict("javascript:alert('xss')"), "alert('xss')");
/// ```
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
/// - No SVG / MathML
///
/// # Exemple
///
/// ```rust
/// use runique::utils::forms::sanitizer::sanitize_rich;
/// // Les balises autorisées sont conservées, les autres supprimées
/// let html = "<b>gras</b> <script>alert('xss')</script>";
/// let result = sanitize_rich(html);
/// assert!(result.contains("<b>gras</b>"));
/// assert!(!result.contains("<script>"));
/// ```
pub fn sanitize_rich(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Build the Builder without conflict with link_rel
    let mut builder = Builder::new();
    let builder = builder.tags(ALLOWED_TAGS.clone());
    let builder = builder.tag_attributes(ALLOWED_ATTRS.clone());
    let mut builder = builder.url_schemes(HashSet::from(["http", "https", "mailto"]));

    // Add link_rel only if rel is not already in ALLOWED_ATTRS
    if !ALLOWED_ATTRS
        .get("a")
        .is_some_and(|attrs| attrs.contains("rel"))
    {
        builder = builder.link_rel(Some("noopener noreferrer"));
    }

    builder.clean(input).to_string().trim().to_string()
}

/// =============================
/// SINGLE ENTRY POINT
/// =============================
///
/// Automatically decides the sanitation mode
///
/// # Exemple
///
/// ```rust
/// use runique::utils::forms::sanitizer::sanitize;
/// // Champ "description" autorise le HTML riche (si présent dans RICH_CONTENT_FIELDS)
/// let html = "<b>gras</b>";
/// let result = sanitize("description", html);
/// // Le résultat dépend de la config du projet (RICH_CONTENT_FIELDS)
/// // Pour un champ non riche, le HTML est supprimé
/// let result2 = sanitize("autre", html);
/// assert!(!result2.contains("<b>"));
/// ```
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
