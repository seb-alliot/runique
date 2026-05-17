//! Trusted proxies configuration passed via closure to the builder.
use crate::middleware::security::trusted_proxies::TrustedProxies;
use std::net::IpAddr;

// ═══════════════════════════════════════════════════════════════
// TrustedProxiesConfig
// ═══════════════════════════════════════════════════════════════
//
// Used exclusively in the `with_trusted_proxies` closure:
//
//   .middleware(|m| {
//       m.with_trusted_proxies(|t| {
//           t.private_networks()
//            .proxy("203.0.113.5")
//       })
//   })
//
// Default: private networks (RFC 1918 + loopback) — covers nginx same-machine,
// Docker networks, Kubernetes clusters without any explicit configuration.
//
// METHODS:
//   .private_networks()           — add 127/8, 10/8, 172.16/12, 192.168/16, ::1, fc00::/7
//   .proxy("1.2.3.4")             — trust a single IP
//   .cidr("10.0.0.0/8")          — trust a CIDR range
//   .none()                       — clear all trusted proxies (direct exposure)
//
// ═══════════════════════════════════════════════════════════════

/// Trusted proxies configuration, passed via closure to `.with_trusted_proxies(|t| { ... })`.
///
/// Starts from the private networks default. Call `.none()` to clear it before
/// adding your own entries.
///
/// # Example — behind nginx on the same machine
/// ```rust,ignore
/// // Default already covers this — no config needed.
/// ```
///
/// # Example — specific cloud proxy IP
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_trusted_proxies(|t| {
///         t.private_networks()
///          .proxy("203.0.113.5")
///     })
/// })
/// ```
///
/// # Example — disable all proxy trust (direct exposure to internet)
/// ```rust,ignore
/// m.with_trusted_proxies(|t| t.none())
/// ```
pub struct TrustedProxiesConfig {
    exact: Vec<IpAddr>,
    cidrs: Vec<(IpAddr, u8)>,
}

impl Default for TrustedProxiesConfig {
    fn default() -> Self {
        Self {
            exact: vec![],
            cidrs: vec![],
        }
        .private_networks()
    }
}

impl TrustedProxiesConfig {
    // ═══════════════════════════════════════════════════
    // CONFIG METHODS
    // ═══════════════════════════════════════════════════

    /// Adds RFC 1918 private networks and loopback as trusted proxies.
    ///
    /// Covers: `127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`,
    /// `192.168.0.0/16`, `::1/128`, `fc00::/7`.
    pub fn private_networks(mut self) -> Self {
        let private: Vec<(IpAddr, u8)> = vec![
            ("127.0.0.0".parse().unwrap(), 8),
            ("10.0.0.0".parse().unwrap(), 8),
            ("172.16.0.0".parse().unwrap(), 12),
            ("192.168.0.0".parse().unwrap(), 16),
            ("::1".parse().unwrap(), 128),
            ("fc00::".parse().unwrap(), 7),
        ];
        for entry in private {
            if !self.cidrs.contains(&entry) {
                self.cidrs.push(entry);
            }
        }
        self
    }

    /// Trusts a specific IP address as a proxy.
    ///
    /// # Example
    /// ```rust,ignore
    /// t.proxy("203.0.113.5")
    /// ```
    pub fn proxy(mut self, ip: impl AsRef<str>) -> Self {
        if let Ok(addr) = ip.as_ref().parse::<IpAddr>()
            && !self.exact.contains(&addr)
        {
            self.exact.push(addr);
        }
        self
    }

    /// Trusts a CIDR range as a proxy source.
    ///
    /// # Example
    /// ```rust,ignore
    /// t.cidr("10.0.0.0/8")
    /// ```
    pub fn cidr(mut self, cidr: impl AsRef<str>) -> Self {
        if let Some((net, prefix)) = parse_cidr(cidr.as_ref()) {
            let entry = (net, prefix);
            if !self.cidrs.contains(&entry) {
                self.cidrs.push(entry);
            }
        }
        self
    }

    /// Clears all trusted proxies (no forwarded headers trusted).
    ///
    /// Use when the application is directly exposed to the internet
    /// with no reverse proxy in front.
    pub fn none(self) -> Self {
        Self {
            exact: vec![],
            cidrs: vec![],
        }
    }

    // ═══════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════

    pub(crate) fn build(self) -> TrustedProxies {
        TrustedProxies::new(self.exact, self.cidrs)
    }
}

fn parse_cidr(s: &str) -> Option<(IpAddr, u8)> {
    let (ip_str, prefix_str) = s.split_once('/')?;
    let ip: IpAddr = ip_str.trim().parse().ok()?;
    let prefix: u8 = prefix_str.trim().parse().ok()?;
    Some((ip, prefix))
}
