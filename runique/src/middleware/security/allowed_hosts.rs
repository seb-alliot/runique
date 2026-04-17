//! `Host` header validation middleware: rejects requests to unauthorized hosts.
use crate::utils::{aliases::AEngine, trad::t};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

/// Host validation policy: allowlist with wildcard support (`.domain.fr`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HostPolicy {
    /// List of allowed hosts (exact or wildcard prefixed by `.`).
    pub allowed_hosts: Vec<String>,
    /// Enables or disables validation. If `false`, all requests pass.
    pub enabled: bool,
}

impl HostPolicy {
    pub fn new(allowed_hosts: Vec<String>, enabled: bool) -> Self {
        Self {
            allowed_hosts,
            enabled,
        }
    }
    #[must_use]
    pub fn is_host_allowed(&self, host: &str) -> bool {
        fn normalize_host(host: &str) -> &str {
            if host.starts_with('[') {
                host.split(']').next().map_or(host, |h| &host[..=h.len()])
            } else {
                host.split(':').next().unwrap_or(host)
            }
        }

        let host = normalize_host(host);

        self.allowed_hosts.iter().any(|allowed_raw| {
            let allowed = allowed_raw.trim();

            if allowed == "*" {
                return true;
            }

            // normalization on allowed side too
            let allowed_host = normalize_host(allowed);

            if let Some(suffix) = allowed_host.strip_prefix('.') {
                host == suffix
                    || (host.ends_with(allowed_host)
                        && host.as_bytes()[host.len().saturating_sub(allowed_host.len())] == b'.')
            } else {
                allowed_host == host
            }
        })
    }

    pub fn validate(&self, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
        let host = match headers.get(header::HOST) {
            Some(h) => h.to_str().unwrap_or("<invalid host header>"),
            None => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    self.make_error_message("<no host>"),
                ));
            }
        };

        if !self.is_host_allowed(host) {
            return Err((StatusCode::BAD_REQUEST, self.make_error_message(host)));
        }
        Ok(())
    }

    pub(crate) fn make_error_message(&self, _host: &str) -> String {
        t("middleware.bad_request").into_owned()
    }
}

pub(crate) async fn allowed_hosts_middleware(
    State(engine): State<AEngine>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !engine.security_hosts.enabled {
        return next.run(request).await;
    }

    // HTTP/2 uses :authority pseudo-header (exposed via request.uri()),
    // not the Host header. Fall back to URI authority when Host is absent.
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .or_else(|| request.uri().authority().map(|a| a.as_str()));

    let host = match host {
        Some(h) => h,
        None => {
            let msg = engine.security_hosts.make_error_message("<no host>");
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    if !engine.security_hosts.is_host_allowed(host) {
        let msg = engine.security_hosts.make_error_message(host);
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let validator = HostPolicy::new(vec!["exemple.com".to_string()], true);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(!validator.is_host_allowed("www.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain() {
        let validator = HostPolicy::new(vec![".exemple.com".to_string()], true);
        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));
        assert!(validator.is_host_allowed("admin.api.exemple.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_all() {
        let validator = HostPolicy::new(vec!["*".to_string()], true);

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("anything.com"));
    }

    #[test]
    fn test_multiple_hosts() {
        let validator = HostPolicy::new(
            vec![
                "exemple.com".to_string(),
                "www.exemple.com".to_string(),
                ".api.exemple.com".to_string(),
            ],
            true,
        );

        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));
        assert!(validator.is_host_allowed("v1.api.exemple.com"));
        assert!(!validator.is_host_allowed("other.exemple.com"));
    }

    #[test]
    fn test_host_with_port() {
        let validator = HostPolicy::new(vec!["exemple.com".to_string()], true);

        assert!(validator.is_host_allowed("exemple.com:8080"));
        assert!(validator.is_host_allowed("exemple.com:443"));
    }

    #[test]
    fn test_disabled_allows_all() {
        // Validation disabled via RUNIQUE_ENABLE_HOST_VALIDATION=false
        // Bypass happens at the middleware level, not in is_host_allowed
        let validator = HostPolicy::new(vec!["exemple.com".to_string()], false);
        assert!(!validator.is_host_allowed("anything.com"));
        assert!(!validator.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_wildcard_subdomain_security() {
        // Test to ensure "malicious-exemple.com" does not match ".exemple.com"
        let validator = HostPolicy::new(vec![".exemple.com".to_string()], true);

        // Should match
        assert!(validator.is_host_allowed("exemple.com"));
        assert!(validator.is_host_allowed("www.exemple.com"));
        assert!(validator.is_host_allowed("api.exemple.com"));

        // Should NOT match (security bug)
        assert!(!validator.is_host_allowed("malicious-exemple.com"));
        assert!(!validator.is_host_allowed("evil-exemple.com"));
        assert!(!validator.is_host_allowed("exemple.com.evil.com"));
    }
}
