use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// Real client IP extracted from the request after proxy chain validation.
///
/// Injected into extensions by `trusted_proxies_middleware`.
/// Access in handlers via `Extension<ClientIp>`.
#[derive(Clone, Debug, Copy)]
pub struct ClientIp(pub IpAddr);

// ─── CIDR helper ─────────────────────────────────────────────────────────────

fn ip_in_cidr(ip: &IpAddr, network: &IpAddr, prefix_len: u8) -> bool {
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            if prefix_len == 0 {
                return true;
            }
            if prefix_len > 32 {
                return false;
            }
            let shift = 32 - prefix_len;
            let mask = if shift == 32 { 0u32 } else { u32::MAX << shift };
            (u32::from_be_bytes(ip.octets()) & mask) == (u32::from_be_bytes(net.octets()) & mask)
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            if prefix_len == 0 {
                return true;
            }
            if prefix_len > 128 {
                return false;
            }
            let shift = 128 - prefix_len;
            let mask = if shift == 128 {
                0u128
            } else {
                u128::MAX << shift
            };
            (u128::from_be_bytes(ip.octets()) & mask) == (u128::from_be_bytes(net.octets()) & mask)
        }
        _ => false,
    }
}

/// Normalizes IPv4-mapped IPv6 addresses (`::ffff:a.b.c.d`) to plain IPv4 so a
/// dual-stack socket's mapped peers still match the IPv4 trusted-proxy CIDRs.
/// Uses `to_ipv4_mapped` (not `to_ipv4`) so genuine IPv6 like `::1` is left intact.
fn canonicalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(v6),
        },
        v4 => v4,
    }
}

// ─── TrustedProxies ──────────────────────────────────────────────────────────

/// Trusted proxy configuration stored on the engine.
///
/// Built via [`TrustedProxiesConfig`](crate::app::staging::TrustedProxiesConfig).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustedProxies {
    /// Exact trusted IPs.
    exact: Vec<IpAddr>,
    /// Trusted CIDR ranges: (network address, prefix length).
    cidrs: Vec<(IpAddr, u8)>,
}

impl Default for TrustedProxies {
    fn default() -> Self {
        // RFC 1918 private networks + loopback — safe to trust as proxies
        Self {
            exact: vec![],
            cidrs: vec![
                ("127.0.0.0".parse().unwrap(), 8),    // IPv4 loopback
                ("10.0.0.0".parse().unwrap(), 8),     // Class A private
                ("172.16.0.0".parse().unwrap(), 12),  // Class B private
                ("192.168.0.0".parse().unwrap(), 16), // Class C private
                ("::1".parse().unwrap(), 128),        // IPv6 loopback
                ("fc00::".parse().unwrap(), 7),       // IPv6 unique local (fc00::/7)
            ],
        }
    }
}

impl TrustedProxies {
    pub fn new(exact: Vec<IpAddr>, cidrs: Vec<(IpAddr, u8)>) -> Self {
        Self { exact, cidrs }
    }

    pub fn is_trusted(&self, ip: &IpAddr) -> bool {
        if self.exact.contains(ip) {
            return true;
        }
        self.cidrs
            .iter()
            .any(|(net, prefix)| ip_in_cidr(ip, net, *prefix))
    }

    /// Extracts the real client IP from the request.
    ///
    /// Algorithm:
    ///   1. If the direct connection IP (conn_ip) is not trusted, return it directly.
    ///   2. Otherwise, parse `X-Forwarded-For` and walk from right to left,
    ///      skipping trusted proxies, to find the first untrusted IP.
    ///   3. If all XFF entries are trusted, return the leftmost (client claim).
    pub fn extract_client_ip(&self, req: &Request<Body>, conn_ip: Option<IpAddr>) -> IpAddr {
        // No socket peer info (tests / non-socket contexts): we cannot verify the
        // request actually transited a trusted proxy, so X-Forwarded-For — which is
        // fully client-controlled — must NOT be trusted. Return loopback without
        // ever reading the header, so a missing ConnectInfo can't enable spoofing.
        let Some(conn_ip_value) = conn_ip else {
            return IpAddr::V4(Ipv4Addr::LOCALHOST);
        };
        let conn_ip_value = canonicalize_ip(conn_ip_value);

        // Direct connection from a non-proxy: the peer IS the client, ignore XFF.
        if !self.is_trusted(&conn_ip_value) {
            return conn_ip_value;
        }

        let xff: Vec<IpAddr> = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                s.split(',')
                    .filter_map(|ip| ip.trim().parse::<IpAddr>().ok())
                    .map(canonicalize_ip)
                    .collect()
            })
            .unwrap_or_default();

        if xff.is_empty() {
            return conn_ip_value;
        }

        // Walk from rightmost (last proxy) to leftmost (client)
        for ip in xff.iter().rev() {
            if !self.is_trusted(ip) {
                return *ip;
            }
        }

        // All entries are trusted proxies — use the leftmost as client claim
        xff.into_iter().next().unwrap_or(conn_ip_value)
    }
}

// ─── Middleware ───────────────────────────────────────────────────────────────

use crate::utils::aliases::AEngine;

pub async fn trusted_proxies_middleware(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    use axum::extract::ConnectInfo;
    use std::net::SocketAddr;

    let conn_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());

    let client_ip = engine.trusted_proxies.extract_client_ip(&req, conn_ip);
    req.extensions_mut().insert(ClientIp(client_ip));
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn req_with_xff(xff: Option<&str>) -> Request<Body> {
        let mut b = Request::builder();
        if let Some(v) = xff {
            b = b.header("x-forwarded-for", v);
        }
        b.body(Body::empty()).unwrap()
    }

    // ── canonicalize_ip ──────────────────────────────────────────────

    #[test]
    fn canonicalize_maps_ipv4_mapped_v6() {
        assert_eq!(canonicalize_ip(ip("::ffff:10.0.0.5")), ip("10.0.0.5"));
        assert_eq!(canonicalize_ip(ip("::ffff:8.8.8.8")), ip("8.8.8.8"));
    }

    #[test]
    fn canonicalize_leaves_plain_ipv4_untouched() {
        assert_eq!(canonicalize_ip(ip("1.2.3.4")), ip("1.2.3.4"));
    }

    #[test]
    fn canonicalize_leaves_genuine_ipv6_untouched() {
        // ::1 must NOT become 0.0.0.1 — that is the `to_ipv4()` trap we avoid.
        assert_eq!(canonicalize_ip(ip("::1")), ip("::1"));
        assert_eq!(canonicalize_ip(ip("2001:db8::1")), ip("2001:db8::1"));
    }

    // ── extract_client_ip: no ConnectInfo (anti-spoofing fallback) ───

    #[test]
    fn no_conn_info_never_trusts_xff() {
        let tp = TrustedProxies::default();
        // A forged XFF must NOT be honored when the peer IP is unknown.
        let req = req_with_xff(Some("9.9.9.9"));
        assert_eq!(tp.extract_client_ip(&req, None), ip("127.0.0.1"));
    }

    // ── extract_client_ip: direct (untrusted) connection ─────────────

    #[test]
    fn direct_untrusted_peer_ignores_xff() {
        let tp = TrustedProxies::default();
        let req = req_with_xff(Some("9.9.9.9")); // spoof attempt
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("8.8.8.8"))),
            ip("8.8.8.8")
        );
    }

    // ── extract_client_ip: behind a trusted proxy ────────────────────

    #[test]
    fn trusted_proxy_uses_xff_client() {
        let tp = TrustedProxies::default();
        let req = req_with_xff(Some("9.9.9.9"));
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("10.0.0.1"))),
            ip("9.9.9.9")
        );
    }

    #[test]
    fn trusted_proxy_walks_right_to_left() {
        let tp = TrustedProxies::default();
        // "<client>, <internal hop>" → real client is the leftmost untrusted entry.
        let req = req_with_xff(Some("9.9.9.9, 10.0.0.2"));
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("10.0.0.1"))),
            ip("9.9.9.9")
        );
    }

    #[test]
    fn trusted_proxy_no_xff_returns_peer() {
        let tp = TrustedProxies::default();
        let req = req_with_xff(None);
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("10.0.0.1"))),
            ip("10.0.0.1")
        );
    }

    #[test]
    fn all_xff_trusted_returns_leftmost() {
        let tp = TrustedProxies::default();
        let req = req_with_xff(Some("10.0.0.9, 10.0.0.2"));
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("10.0.0.1"))),
            ip("10.0.0.9")
        );
    }

    // ── dual-stack: IPv4-mapped peer / XFF entries ───────────────────

    #[test]
    fn ipv4_mapped_proxy_is_recognized_as_trusted() {
        let tp = TrustedProxies::default();
        // Dual-stack socket: the 10.0.0.1 proxy arrives as ::ffff:10.0.0.1.
        // Before canonicalization this was seen as untrusted → XFF dropped.
        let req = req_with_xff(Some("9.9.9.9"));
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("::ffff:10.0.0.1"))),
            ip("9.9.9.9")
        );
    }

    #[test]
    fn ipv4_mapped_xff_entry_is_canonicalized() {
        let tp = TrustedProxies::default();
        let req = req_with_xff(Some("::ffff:9.9.9.9"));
        assert_eq!(
            tp.extract_client_ip(&req, Some(ip("10.0.0.1"))),
            ip("9.9.9.9")
        );
    }
}
