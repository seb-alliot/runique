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

#[test]
fn test_is_localhost_sans_port() {
    assert!(is_localhost(&build_with_host("localhost")));
}

#[test]
fn test_is_localhost_127_sans_port() {
    assert!(is_localhost(&build_with_host("127.0.0.1")));
}

#[test]
fn test_is_localhost_domaine_ressemble() {
    // "notlocalhost.com" ne doit pas matcher
    assert!(!is_localhost(&build_with_host("notlocalhost.com")));
}

#[test]
fn test_is_localhost_ip_externe() {
    assert!(!is_localhost(&build_with_host("192.168.1.1")));
}

#[test]
fn test_is_localhost_ip_externe_avec_port() {
    assert!(!is_localhost(&build_with_host("10.0.0.1:8080")));
}

#[test]
fn test_is_localhost_domaine_production() {
    assert!(!is_localhost(&build_with_host("monapp.com")));
}
