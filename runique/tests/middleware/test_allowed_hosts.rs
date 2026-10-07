// Tests pour allowed_hosts middleware

use axum::http::{HeaderMap, StatusCode, header};
use runique::middleware::security::allowed_hosts::HostPolicy;

#[test]
fn test_host_policy_basic() {
    let policy = HostPolicy::new(vec!["localhost".to_string()], false);
    assert!(policy.is_host_allowed("localhost"));
    assert!(!policy.is_host_allowed("evil.com"));
}

#[test]
fn test_ipv6_host_without_closing_bracket_does_not_panic() {
    let policy = HostPolicy::new(vec!["[::1]".to_string()], false);
    assert!(!policy.is_host_allowed("[::1"));
    assert!(!policy.is_host_allowed("["));
    assert!(policy.is_host_allowed("[::1]:8080"));
}

#[test]
fn test_exact_match() {
    let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);
    assert!(validator.is_host_allowed("exemple.com"));
    assert!(!validator.is_host_allowed("www.exemple.com"));
    assert!(!validator.is_host_allowed("malicious.com"));
}

#[test]
fn test_wildcard_subdomain() {
    let validator = HostPolicy::new(vec![".exemple.com".to_string()], false);
    assert!(validator.is_host_allowed("exemple.com"));
    assert!(validator.is_host_allowed("www.exemple.com"));
    assert!(validator.is_host_allowed("api.exemple.com"));
    assert!(validator.is_host_allowed("admin.api.exemple.com"));
    assert!(!validator.is_host_allowed("malicious.com"));
}

#[test]
fn test_wildcard_all() {
    let validator = HostPolicy::new(vec!["*".to_string()], false);
    assert!(validator.is_host_allowed("exemple.com"));
    assert!(validator.is_host_allowed("n-importe-quoi.com"));
}

#[test]
fn test_multiple_hosts() {
    let validator = HostPolicy::new(
        vec![
            "exemple.com".to_string(),
            "www.exemple.com".to_string(),
            ".api.exemple.com".to_string(),
        ],
        false,
    );
    assert!(validator.is_host_allowed("exemple.com"));
    assert!(validator.is_host_allowed("www.exemple.com"));
    assert!(validator.is_host_allowed("api.exemple.com"));
    assert!(validator.is_host_allowed("v1.api.exemple.com"));
    assert!(!validator.is_host_allowed("autre.exemple.com"));
}

#[test]
fn test_host_with_port() {
    let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);
    assert!(validator.is_host_allowed("exemple.com:8080"));
    assert!(validator.is_host_allowed("exemple.com:443"));
}

#[test]
fn test_disabled_bypasses_at_middleware_level() {
    // Le bypass se fait dans le middleware (enabled=false → skip),
    // pas dans is_host_allowed. Avec enabled=false, la validation reste active.
    let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);
    assert!(!validator.is_host_allowed("n-importe-quoi.com"));
    assert!(!validator.is_host_allowed("malicious.com"));
    // L'hôte autorisé est toujours accepté
    assert!(validator.is_host_allowed("exemple.com"));
}

#[test]
fn test_wildcard_subdomain_security() {
    // Test pour éviter que "malicious-exemple.com" match ".exemple.com"
    let validator = HostPolicy::new(vec![".exemple.com".to_string()], false);
    assert!(validator.is_host_allowed("exemple.com"));
    assert!(validator.is_host_allowed("www.exemple.com"));
    assert!(validator.is_host_allowed("api.exemple.com"));
    assert!(!validator.is_host_allowed("malicious-exemple.com"));
    assert!(!validator.is_host_allowed("evil-exemple.com"));
    assert!(!validator.is_host_allowed("exemple.com.evil.com"));
}

#[test]
fn test_validate_ok_and_error() {
    let validator = HostPolicy::new(vec!["localhost".to_string()], false);
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, "localhost".parse().unwrap());
    assert!(validator.validate(&headers).is_ok());

    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, "evil.com".parse().unwrap());
    let res = validator.validate(&headers);
    assert!(res.is_err());
    if let Err((status, msg)) = res {
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(msg.contains("Bad Request"));
    }
}

#[test]
fn test_validate_no_host_header() {
    let validator = HostPolicy::new(vec!["localhost".to_string()], true);
    let headers = HeaderMap::new();
    let res = validator.validate(&headers);
    assert!(res.is_err());
    if let Err((status, _msg)) = res {
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[test]
fn test_make_error_message() {
    // make_error_message retourne la traduction middleware.bad_request ("Bad Request")
    // sans exposer l'hôte reçu (protection contre l'énumération d'hôtes)
    let validator = HostPolicy::new(vec!["localhost".to_string()], true);
    let msg = validator.validate(&HeaderMap::new()).err().unwrap().1;
    assert!(msg.contains("Bad Request"));
}

// Written from cargo-mutants survivors (2026-10-07): a `.domain` entry only
// accepts hosts that END with it — a `.` at the right offset is not enough.
#[test]
fn test_subdomain_entry_needs_the_suffix_not_just_a_dot() {
    let policy = HostPolicy::new(vec![".exemple.com".to_string()], false);
    // 13 chars, with a `.` exactly where ".exemple.com" (12) would start.
    assert!(!policy.is_host_allowed("a.bcdefghijkl"));
    assert!(policy.is_host_allowed("www.exemple.com"));
}
