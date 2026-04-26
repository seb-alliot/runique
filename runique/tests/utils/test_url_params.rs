//! Tests — utils/config/url_params
//! Couvre : UrlParams::new, UrlParams::get (path priority, query fallback, missing)

use runique::utils::config::url_params::UrlParams;
use std::collections::HashMap;

fn hmap(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn test_get_from_path() {
    let path = hmap(&[("id", "42")]);
    let query = hmap(&[]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("id"), Some("42"));
}

#[test]
fn test_get_from_query_fallback() {
    let path = hmap(&[]);
    let query = hmap(&[("page", "3")]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("page"), Some("3"));
}

#[test]
fn test_path_has_priority_over_query() {
    let path = hmap(&[("slug", "from-path")]);
    let query = hmap(&[("slug", "from-query")]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("slug"), Some("from-path"));
}

#[test]
fn test_missing_key_returns_none() {
    let path = hmap(&[]);
    let query = hmap(&[]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("unknown"), None);
}

#[test]
fn test_multiple_keys() {
    let path = hmap(&[("id", "10")]);
    let query = hmap(&[("sort", "asc"), ("page", "2")]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("id"), Some("10"));
    assert_eq!(params.get("sort"), Some("asc"));
    assert_eq!(params.get("page"), Some("2"));
    assert_eq!(params.get("missing"), None);
}

#[test]
fn test_empty_value() {
    let path = hmap(&[("key", "")]);
    let query = hmap(&[]);
    let params = UrlParams::new(&path, &query);
    assert_eq!(params.get("key"), Some(""));
}
