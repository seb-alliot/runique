//! Tests — utils/constante/parse.rs
//! Couvre : CONSTRAINT_REGEX, KEY_REGEX, FAILED_REGEX, FOR_KEY_REGEX,
//!          RICH_CONTENT_FIELDS, ALLOWED_TAGS, ALLOWED_ATTRS

use runique::utils::constante::parse::{
    ALLOWED_ATTRS, ALLOWED_TAGS, CONSTRAINT_REGEX, FAILED_REGEX, FOR_KEY_REGEX, KEY_REGEX,
    RICH_CONTENT_FIELDS,
};

// ═══════════════════════════════════════════════════════════════
// CONSTRAINT_REGEX
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_constraint_regex_simple() {
    let captures = CONSTRAINT_REGEX
        .captures("constraint users_email_key")
        .unwrap()
        .unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("users_email_key"));
}

#[test]
fn test_constraint_regex_with_quotes() {
    let captures = CONSTRAINT_REGEX
        .captures(r#"constraint "users_email_key""#)
        .unwrap()
        .unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("users_email_key"));
}

#[test]
fn test_constraint_regex_no_match() {
    let result = CONSTRAINT_REGEX.captures("hello world").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_constraint_regex_with_unique() {
    let captures = CONSTRAINT_REGEX
        .captures("contrainte unique my_constraint")
        .unwrap()
        .unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("my_constraint"));
}

// ═══════════════════════════════════════════════════════════════
// KEY_REGEX
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_key_regex_single_column() {
    let captures = KEY_REGEX.captures("Key (email)").unwrap().unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("email"));
}

#[test]
fn test_key_regex_multi_column() {
    let captures = KEY_REGEX.captures("Key (user_id, email)").unwrap().unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("user_id, email"));
}

#[test]
fn test_key_regex_no_match() {
    let result = KEY_REGEX.captures("no key here").unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// FAILED_REGEX
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_failed_regex_matches_field() {
    let captures = FAILED_REGEX
        .captures("failed: users.email")
        .unwrap()
        .unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("email"));
}

#[test]
fn test_failed_regex_matches_underscore_field() {
    let captures = FAILED_REGEX
        .captures("failed: my_table.user_name_unique")
        .unwrap()
        .unwrap();
    assert_eq!(
        captures.get(1).map(|m| m.as_str()),
        Some("user_name_unique")
    );
}

#[test]
fn test_failed_regex_no_match() {
    let result = FAILED_REGEX.captures("no error here").unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// FOR_KEY_REGEX
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_for_key_regex_matches() {
    let captures = FOR_KEY_REGEX
        .captures("for key 'users.email'")
        .unwrap()
        .unwrap();
    assert_eq!(captures.get(1).map(|m| m.as_str()), Some("email"));
}

#[test]
fn test_for_key_regex_underscore_key() {
    let captures = FOR_KEY_REGEX
        .captures("for key 'accounts.account_email_unique'")
        .unwrap()
        .unwrap();
    assert_eq!(
        captures.get(1).map(|m| m.as_str()),
        Some("account_email_unique")
    );
}

#[test]
fn test_for_key_regex_no_match() {
    let result = FOR_KEY_REGEX.captures("no key here").unwrap();
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// RICH_CONTENT_FIELDS
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_rich_content_fields_includes_known() {
    assert!(RICH_CONTENT_FIELDS.contains("content"));
    assert!(RICH_CONTENT_FIELDS.contains("description"));
    assert!(RICH_CONTENT_FIELDS.contains("bio"));
    assert!(RICH_CONTENT_FIELDS.contains("comment"));
    assert!(RICH_CONTENT_FIELDS.contains("message"));
    assert!(RICH_CONTENT_FIELDS.contains("body"));
    assert!(RICH_CONTENT_FIELDS.contains("post"));
    assert!(RICH_CONTENT_FIELDS.contains("article"));
}

#[test]
fn test_rich_content_fields_excludes_non_rich() {
    assert!(!RICH_CONTENT_FIELDS.contains("username"));
    assert!(!RICH_CONTENT_FIELDS.contains("email"));
    assert!(!RICH_CONTENT_FIELDS.contains("password"));
    assert!(!RICH_CONTENT_FIELDS.contains("id"));
}

// ═══════════════════════════════════════════════════════════════
// ALLOWED_TAGS
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_allowed_tags_includes_safe_html() {
    assert!(ALLOWED_TAGS.contains("p"));
    assert!(ALLOWED_TAGS.contains("strong"));
    assert!(ALLOWED_TAGS.contains("em"));
    assert!(ALLOWED_TAGS.contains("a"));
    assert!(ALLOWED_TAGS.contains("ul"));
    assert!(ALLOWED_TAGS.contains("li"));
    assert!(ALLOWED_TAGS.contains("h1"));
    assert!(ALLOWED_TAGS.contains("code"));
}

#[test]
fn test_allowed_tags_excludes_dangerous() {
    assert!(!ALLOWED_TAGS.contains("script"));
    assert!(!ALLOWED_TAGS.contains("iframe"));
    assert!(!ALLOWED_TAGS.contains("object"));
    assert!(!ALLOWED_TAGS.contains("embed"));
    assert!(!ALLOWED_TAGS.contains("form"));
    assert!(!ALLOWED_TAGS.contains("input"));
}

// ═══════════════════════════════════════════════════════════════
// ALLOWED_ATTRS
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_allowed_attrs_a_tag_has_href() {
    let a_attrs = ALLOWED_ATTRS
        .get("a")
        .expect("tag 'a' doit avoir des attributs");
    assert!(a_attrs.contains("href"));
    assert!(a_attrs.contains("title"));
    assert!(a_attrs.contains("target"));
}

#[test]
fn test_allowed_attrs_a_tag_excludes_dangerous() {
    let a_attrs = ALLOWED_ATTRS.get("a").unwrap();
    assert!(!a_attrs.contains("onclick"));
    assert!(!a_attrs.contains("onmouseover"));
    assert!(!a_attrs.contains("style"));
}

#[test]
fn test_allowed_attrs_unknown_tag_returns_none() {
    assert!(ALLOWED_ATTRS.get("script").is_none());
    assert!(ALLOWED_ATTRS.get("div").is_none());
}
