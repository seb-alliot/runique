//! Tests — middleware/security/trusted_proxies.rs : CIDR matching.
//! Written from cargo-mutants survivors (2026-10-07): IPv6 ranges and the
//! exact-prefix boundaries were never checked.

use runique::middleware::TrustedProxies;
use std::net::IpAddr;

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

fn trusts(cidr: (&str, u8), candidate: &str) -> bool {
    TrustedProxies::new(Vec::new(), vec![(ip(cidr.0), cidr.1)]).is_trusted(&ip(candidate))
}

#[test]
fn ipv4_prefix_boundaries() {
    assert!(trusts(("10.1.2.3", 32), "10.1.2.3"), "/32 matches itself");
    assert!(!trusts(("10.1.2.3", 32), "10.1.2.4"));
    assert!(trusts(("192.168.1.0", 24), "192.168.1.255"));
    assert!(!trusts(("192.168.1.0", 24), "192.168.2.1"));
    assert!(trusts(("0.0.0.0", 0), "8.8.8.8"), "/0 matches everything");
    assert!(
        !trusts(("10.0.0.0", 33), "10.0.0.0"),
        "an impossible prefix trusts nothing"
    );
}

#[test]
fn ipv6_prefix_boundaries() {
    assert!(trusts(("::1", 128), "::1"), "/128 matches itself");
    assert!(!trusts(("::1", 128), "::2"));
    assert!(
        trusts(("fc00::", 7), "fd12:3456::1"),
        "unique local fc00::/7"
    );
    assert!(!trusts(("fc00::", 7), "fe80::1"));
    assert!(trusts(("2001:db8::", 32), "2001:db8:ffff::1"));
    assert!(!trusts(("2001:db8::", 32), "2001:db9::1"));
    assert!(trusts(("::", 0), "2001:db8::1"), "/0 matches everything");
    assert!(
        !trusts(("::1", 129), "::1"),
        "an impossible prefix trusts nothing"
    );
}

#[test]
fn an_ipv4_range_never_trusts_an_ipv6_address() {
    assert!(!trusts(("0.0.0.0", 0), "::1"));
    assert!(!trusts(("::", 0), "127.0.0.1"));
}
