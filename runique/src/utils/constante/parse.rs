use fancy_regex::Regex;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

pub static CONSTRAINT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:constraint|contrainte\s+unique)\s+(?:["«])?([a-zA-Z0-9_]+)(?:["»])?"#).unwrap()
});

pub static KEY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"Key\s+\(([^)]+)\)").unwrap());

pub static FAILED_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"failed:\s+[a-zA-Z0-9_]+\.([a-zA-Z0-9_]+)").unwrap());

pub static FOR_KEY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"for\s+key\s+'[^.]+\.([^']+)'").unwrap());

/// Fields allowed to contain rich HTML
pub static RICH_CONTENT_FIELDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
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

/// HTML Policy: allowed tags (security)
pub static ALLOWED_TAGS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "p",
        "br",
        "strong",
        "em",
        "b",
        "i",
        "ul",
        "ol",
        "li",
        "blockquote",
        "code",
        "pre",
        "a",
        "h1",
        "h2",
        "h3",
        "h4",
    ])
});

/// HTML Policy: allowed attributes per tag
pub static ALLOWED_ATTRS: Lazy<HashMap<&'static str, HashSet<&'static str>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("a", HashSet::from(["href", "title", "target"]));
    map
});
