//! Tests — utils/forms/parse_boolean
//! Couvre : parse_bool

use runique::utils::parse_bool;
use std::collections::HashMap;

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn test_parse_bool_on() {
    let data = map(&[("active", "on")]);
    assert!(parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_true() {
    let data = map(&[("active", "true")]);
    assert!(parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_one() {
    let data = map(&[("active", "1")]);
    assert!(parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_false_string() {
    let data = map(&[("active", "false")]);
    assert!(!parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_zero() {
    let data = map(&[("active", "0")]);
    assert!(!parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_empty_string() {
    let data = map(&[("active", "")]);
    assert!(!parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_missing_key() {
    let data = map(&[("other", "on")]);
    assert!(!parse_bool(&data, "active"));
}

#[test]
fn test_parse_bool_off() {
    let data = map(&[("active", "off")]);
    assert!(!parse_bool(&data, "active"));
}
