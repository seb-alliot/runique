use crate::utils::{aliases::AEngine, runique_log::get_log};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn open_redirect_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let response = next.run(req).await;

    if !response.status().is_redirection() {
        return response;
    }

    let Some(location) = response.headers().get(header::LOCATION) else {
        return response;
    };

    let Ok(location_str) = location.to_str() else {
        return response;
    };

    if is_safe_redirect(location_str, &engine) {
        return response;
    }

    if let Some(level) = get_log().host_validation {
        crate::runique_log!(level, location = %location_str, "open redirect blocked");
    }

    (StatusCode::BAD_REQUEST, "Forbidden redirect").into_response()
}

fn is_safe_redirect(location: &str, engine: &crate::engine::RuniqueEngine) -> bool {
    // Relative path (not protocol-relative) — same origin, always safe
    if location.starts_with('/') && !location.starts_with("//") {
        return true;
    }

    // Extract host from absolute or protocol-relative URL
    let host = extract_host(location);

    let Some(host) = host else {
        // Unparseable — treat as unsafe
        return false;
    };

    // Localhost destinations are always safe (unreachable by external attackers)
    if is_local_host(host) {
        return true;
    }

    // Check against the configured allowed hosts
    engine.security_hosts.is_host_allowed(host)
}

pub fn extract_host(location: &str) -> Option<&str> {
    // Strip scheme: "https://host/path" or "//host/path"
    let without_scheme = if let Some(rest) = location.strip_prefix("//") {
        rest
    } else {
        location
            .strip_prefix("http://")
            .or_else(|| location.strip_prefix("https://"))?
    };

    // Host ends at the first '/', '?', '#', or end of string
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .filter(|h| !h.is_empty())?;

    Some(host)
}

pub fn is_local_host(host: &str) -> bool {
    // IPv6: "[addr]" or "[addr]:port"
    if host.starts_with('[') {
        let addr = host.split(']').next().map(|s| &s[1..]).unwrap_or(host);
        return is_loopback_ipv6(addr);
    }
    // IPv4 / hostname: strip optional port
    let bare = host.split(':').next().unwrap_or(host);
    bare == "localhost" || bare.starts_with("127.")
}

fn is_loopback_ipv6(addr: &str) -> bool {
    // Short form ::1
    if addr == "::1" {
        return true;
    }
    // Full form 0:0:0:0:0:0:0:1
    if addr == "0:0:0:0:0:0:0:1" {
        return true;
    }
    // IPv4-mapped ::ffff:127.x.x.x or ::ffff:7fxx:xxxx
    if let Some(rest) = addr.to_ascii_lowercase().strip_prefix("::ffff:") {
        // Dotted notation: ::ffff:127.x.x.x
        if rest.starts_with("127.") {
            return true;
        }
        // Hex notation: ::ffff:7f00:0001 etc. — first group starts with 7f
        if rest.starts_with("7f") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://evil.com/path"), Some("evil.com"));
        assert_eq!(
            extract_host("http://evil.com:8080/path"),
            Some("evil.com:8080")
        );
        assert_eq!(extract_host("//evil.com/path"), Some("evil.com"));
        assert_eq!(extract_host("/relative"), None);
        assert_eq!(extract_host("https://evil.com"), Some("evil.com"));
        assert_eq!(extract_host("https://evil.com?q=1"), Some("evil.com"));
    }

    #[test]
    fn test_is_local_host() {
        // hostname
        assert!(is_local_host("localhost"));
        assert!(is_local_host("localhost:8080"));
        assert!(!is_local_host("notlocalhost.com"));
        assert!(!is_local_host("localhost.evil.com"));

        // IPv4 loopback range 127.0.0.0/8
        assert!(is_local_host("127.0.0.1"));
        assert!(is_local_host("127.0.0.1:3000"));
        assert!(is_local_host("127.0.0.2"));
        assert!(is_local_host("127.1.2.3"));
        assert!(is_local_host("127.255.255.255"));
        assert!(!is_local_host("128.0.0.1"));

        // IPv6 loopback
        assert!(is_local_host("[::1]"));
        assert!(is_local_host("[::1]:8080"));
        assert!(is_local_host("[0:0:0:0:0:0:0:1]"));
        assert!(is_local_host("[0:0:0:0:0:0:0:1]:443"));

        // IPv4-mapped IPv6 loopback
        assert!(is_local_host("[::ffff:127.0.0.1]"));
        assert!(is_local_host("[::ffff:7f00:1]"));
        assert!(is_local_host("[::FFFF:127.0.0.1]")); // uppercase

        assert!(!is_local_host("evil.com"));
        assert!(!is_local_host("[::2]"));
    }
}
