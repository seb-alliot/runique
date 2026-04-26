// Tests pour dev_no_cache_middleware (structure)

use crate::helpers::request::build_with_host;
use runique::middleware::dev::cache::is_localhost;

#[test]
fn test_is_localhost_true() {
    assert!(is_localhost(&build_with_host("localhost:3000")));
}

#[test]
fn test_is_localhost_127() {
    assert!(is_localhost(&build_with_host("127.0.0.1:8080")));
}

#[test]
fn test_is_localhost_ipv6() {
    assert!(is_localhost(&build_with_host("[::1]:3000")));
}

#[test]
fn test_is_localhost_false() {
    assert!(!is_localhost(&build_with_host("evil.com")));
}
