//! Allowed hosts configuration passed via closure to the builder.
//
// Used exclusively in the `with_allowed_hosts` closure:
//
//   .middleware(|m| {
//       m.with_allowed_hosts(|h| {
//           h.enabled(true)
//            .host("mysite.com")
//            .host("www.mysite.com")
//       })
//   })
//
// ACTIVATION:
//   .enabled(true)          → enables Host header validation
//   .enabled(false)         → disables (rare case, escape hatch)
//
// HOSTS:
//   .host("mysite.com")     → adds an exact host
//   .hosts(vec![...])       → adds multiple hosts at once
//
// WILDCARDS:
//   ".mysite.com"           → mysite.com + all subdomains
//   "*"                     → allow everything (effectively disables validation)
//
// ═══════════════════════════════════════════════════════════════

/// Allowed hosts configuration, passed via closure to `.with_allowed_hosts(|h| { ... })`.
///
/// Disabled by default — call `.enabled(true)` to enable.
///
/// # Example
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_allowed_hosts(|h| {
///         h.enabled(true)
///          .host("mysite.com")
///          .host("www.mysite.com")
///     })
/// })
/// ```
///
/// # Wildcard subdomains
/// ```rust,ignore
/// m.with_allowed_hosts(|h| {
///     h.enabled(true)
///      .host(".mysite.com") // mysite.com + subdomains
/// })
/// ```
///
/// # Disable — do not call `.with_allowed_hosts` at all.
#[derive(Default)]
pub struct HostConfig {
    pub(crate) hosts: Vec<String>,
    pub(crate) enabled: bool,
}

impl HostConfig {
    /// Enables or disables Host header validation.
    ///
    /// Without calling `.enabled(true)`, validation is inactive
    /// even if hosts are defined.
    pub fn enabled(mut self, enable: bool) -> Self {
        self.enabled = enable;
        self
    }

    /// Adds an allowed host.
    ///
    /// Can be chained multiple times:
    /// ```rust,ignore
    /// h.host("mysite.com").host("www.mysite.com")
    /// ```
    ///
    /// Prefix with `.` to allow the domain and all its subdomains:
    /// ```rust,ignore
    /// h.host(".mysite.com") // mysite.com + api.mysite.com + ...
    /// ```
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.hosts.push(host.into());
        self
    }

    /// Adds multiple hosts at once.
    ///
    /// ```rust,ignore
    /// h.hosts(vec!["mysite.com", "www.mysite.com"])
    /// ```
    pub fn hosts(mut self, hosts: Vec<impl Into<String>>) -> Self {
        self.hosts.extend(hosts.into_iter().map(Into::into));
        self
    }
}
