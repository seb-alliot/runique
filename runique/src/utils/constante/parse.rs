//! Parsing constants — DB constraint regex, HTML whitelist (ammonia), rich-text fields.
use fancy_regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

pub static CONSTRAINT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:constraint|contrainte\s+unique)\s+(?:["«])?([a-zA-Z0-9_]+)(?:["»])?"#).unwrap()
});

pub static KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Key\s+\(([^)]+)\)").unwrap());

pub static FAILED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"failed:\s+[a-zA-Z0-9_]+\.([a-zA-Z0-9_]+)").unwrap());

pub static FOR_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"for\s+key\s+'[^.]+\.([^']+)'").unwrap());

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
