use crate::utils::{aliases::AEngine, runique_log::get_log};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Blocks redirects to untrusted destinations: inspects the `Location`
/// header of a redirect response and the `Refresh` header of any response,
/// and replaces the response with a `400 Bad Request` unless
/// `is_safe_redirect` accepts every destination (same-origin relative path,
/// the client's own loopback, or a host in the configured allowlist).
pub async fn open_redirect_middleware(
    State(engine): State<AEngine>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let response = next.run(req).await;
    let headers = response.headers();

    // A header may hold bytes `to_str` refuses (non-ASCII): read it lossily
    // rather than letting it through unchecked — a browser still follows it.
    let mut targets: Vec<String> = Vec::new();
    if response.status().is_redirection()
        && let Some(location) = headers.get(header::LOCATION)
    {
        targets.push(String::from_utf8_lossy(location.as_bytes()).into_owned());
    }
    // `Refresh` sends the browser elsewhere too, whatever the status.
    if let Some(refresh) = headers.get(header::REFRESH) {
        targets.extend(refresh_targets(&String::from_utf8_lossy(refresh.as_bytes())));
    }

    let Some(unsafe_target) = targets.iter().find(|t| !is_safe_redirect(t, &engine)) else {
        return response;
    };

    if let Some(level) = get_log()
        .middleware
        .as_ref()
        .and_then(|m| m.host_validation)
    {
        crate::runique_log!(level, location = %unsafe_target, "open redirect blocked");
    }

    (StatusCode::BAD_REQUEST, "Forbidden redirect").into_response()
}

/// The URLs a `Refresh` value may send the browser to (`5; url=/next`,
/// `0;URL='/x'`, `0, /x`, `0; /x`); empty when it only reloads the page. The
/// syntax is loose, so both readings of a leading `url` are returned: with it
/// as the `url=` keyword, and as the start of the URL itself — the redirect is
/// refused if either one leaves the site.
fn refresh_targets(value: &str) -> Vec<String> {
    let rest = value
        .trim_start()
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim_start();
    let rest = rest.strip_prefix([';', ',']).unwrap_or(rest).trim_start();

    let mut readings = vec![rest];
    if let Some(prefix) = rest.get(..3)
        && prefix.eq_ignore_ascii_case("url")
    {
        let after = rest[3..].trim_start();
        readings.push(after.strip_prefix('=').unwrap_or(after).trim_start());
    }

    readings
        .into_iter()
        .map(|reading| match reading.chars().next() {
            Some(quote @ ('\'' | '"')) => reading[1..].split(quote).next().unwrap_or(""),
            _ => reading,
        })
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string)
        .collect()
}

/// Stands in for the current site when resolving a relative `Location`.
/// `.invalid` is reserved (RFC 2606): no real redirect can target it.
const PROBE_HOST: &str = "runique.invalid";

fn is_safe_redirect(location: &str, engine: &crate::engine::RuniqueEngine) -> bool {
    // Resolved by the WHATWG URL parser, the one browsers follow: tabs and
    // newlines dropped, `\` read as `/`, credentials split from the host. A
    // hand-written split misses some of these ("/\t/evil.com" lands on evil.com).
    let Ok(base) = url::Url::parse(&format!("http://{PROBE_HOST}/")) else {
        return false;
    };
    let Ok(target) = base.join(location) else {
        return false;
    };
    if !matches!(target.scheme(), "http" | "https") {
        return false;
    }
    let Some(host_name) = target.host_str() else {
        return false;
    };

    // Same origin: a relative path, resolved against the current site.
    if host_name == PROBE_HOST {
        return true;
    }

    let host_with_port = match target.port() {
        Some(port) => format!("{host_name}:{port}"),
        None => host_name.to_string(),
    };
    let host = host_with_port.as_str();

    // Localhost destinations are allowed to support local development flows.
    // Note: this is the *victim's* loopback, not the server's — redirecting there
    // is an unusual vector (local services on the client). Accepted as a deliberate
    // DX tradeoff; the phishing-to-attacker-domain case stays blocked by the host
    // whitelist below.
    if is_local_host(host) {
        return true;
    }

    // Check against the configured allowed hosts
    engine.security_hosts.is_host_allowed(host)
}

/// Extracts the host (with port, if any) from an absolute or
/// protocol-relative URL (`https://host/path` or `//host/path`). Returns
/// `None` for a relative path, an unparseable value, or credentials in the
/// authority: a browser sends `https://site.com:x@evil.com` to `evil.com`,
/// while the host up to the first `:` reads `site.com`.
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
        .filter(|h| !h.is_empty() && !h.contains('@'))?;

    Some(host)
}

/// Returns `true` if `host` (optionally with a port) is a loopback address:
/// `localhost`, an IPv4 address in `127.0.0.0/8`, or an IPv6 loopback
/// (`::1`, its expanded form, or an IPv4-mapped loopback).
pub fn is_local_host(host: &str) -> bool {
    // IPv6: "[addr]" or "[addr]:port"
    if host.starts_with('[') {
        let addr = host.split(']').next().map(|s| &s[1..]).unwrap_or(host);
        return is_loopback_ipv6(addr);
    }
    // IPv4 / hostname: strip optional port
    let bare = host.split(':').next().unwrap_or(host);
    // Parsed, not a `127.` prefix: `127.evil.com` is a domain anyone can register.
    bare == "localhost"
        || bare
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.is_loopback())
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
        assert!(!is_local_host("127.evil.com"));
        assert!(!is_local_host("127.0.0.1.evil.com:443"));

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
