//! `SecurityPolicies`: what the security middlewares read from the engine.
use crate::utils::aliases::{
    ACsrfExemptPath, APermissionsPolicy, ASecurityCsp, ASecurityHosts, ATrustedProxies,
};

/// The security policies built once at startup and read on every request.
#[derive(Debug, Default)]
pub struct SecurityPolicies {
    /// Active Content Security Policy.
    pub csp: ASecurityCsp,
    /// Policy for validating allowed hosts.
    pub hosts: ASecurityHosts,
    /// Paths exempt from CSRF validation (ex: webhook endpoints).
    pub csrf_exempt_paths: ACsrfExemptPath,
    /// Active Permissions-Policy header configuration.
    pub permissions_policy: APermissionsPolicy,
    /// Trusted proxy IPs/CIDRs for real client IP extraction.
    pub trusted_proxies: ATrustedProxies,
}
