//! The constants of the security primitives, and what the HTML sanitizer lets
//! through: the fields allowed rich content, the tags and the attributes kept.
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Session key storing the CSRF token for the session; also the name of the
/// hidden form field / header carrying it back on submit.
pub const CSRF_TOKEN_KEY: &str = "csrf_token";

/// Fields allowed to contain rich HTML
pub static RICH_CONTENT_FIELDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "content",
        "description",
        "bio",
        "comment",
        "message",
        "body",
        "post",
        "article",
    ])
});

/// HTML Policy: allowed tags (security).
/// Structural/text tags only — no script-bearing or interactive elements.
/// Covers both rich-text fields and Markdown output (tables, headings, etc.).
pub static ALLOWED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // text & inline
        "p",
        "br",
        "hr",
        "strong",
        "em",
        "b",
        "i",
        "u",
        "del",
        "s",
        "sub",
        "sup",
        "code",
        "a",
        "img",
        // headings
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        // blocks & lists
        "ul",
        "ol",
        "li",
        "blockquote",
        "pre",
        // tables (Markdown ENABLE_TABLES)
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
    ])
});

/// HTML Policy: allowed attributes per tag.
/// No `style` (CSS-based vectors) and no event handlers — only inert metadata.
pub static ALLOWED_ATTRS: LazyLock<HashMap<&'static str, HashSet<&'static str>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("a", HashSet::from(["href", "title", "target"]));
        map.insert("img", HashSet::from(["src", "alt", "title"]));
        // `class` is inert (no JS execution) — kept for `language-*` syntax-highlight hints.
        map.insert("code", HashSet::from(["class"]));
        map
    });
