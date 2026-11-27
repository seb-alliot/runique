//! Allowed hosts configuration passed via closure to the builder.
//
// Used exclusively in the `with_allowed_hosts` closure:
//
//   .middleware(|m| {
//       m.with_allowed_hosts(|h| {
//           h.host("mysite.com")
//            .host("www.mysite.com")
//       })
//   })
//
// ACTIVATION:
//   default                 → on in production (RUNIQUE_ENV), off in development
//   .enabled(true)          → always on, to try the validation in development
//   .enabled(false)         → always off (rare case, escape hatch)
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
/// Validation is on in production as soon as a host is listed, off in
/// development (`RUNIQUE_ENV`).
///
/// # Example
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_allowed_hosts(|h| {
///         h.host("mysite.com")
///          .host("www.mysite.com")
///     })
/// })
/// ```
///
/// # Wildcard subdomains
/// ```rust,ignore
/// m.with_allowed_hosts(|h| {
///     h.host(".mysite.com") // mysite.com + subdomains
/// })
/// ```
///
/// # Disable — do not call `.with_allowed_hosts` at all.
#[derive(Default)]
pub struct HostConfig {
    pub(crate) hosts: Vec<String>,
    pub(crate) enabled: Option<bool>,
}

impl HostConfig {
    /// Forces Host header validation on or off, whatever `RUNIQUE_ENV` says.
    pub fn enabled(mut self, enable: bool) -> Self {
        self.enabled = Some(enable);
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
