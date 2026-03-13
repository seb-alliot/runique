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
