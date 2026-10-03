//! `resolve_og_image` — written from cargo-mutants survivors (2026-10-02).
use runique::middleware::allowed_hosts::HostPolicy;
use runique::utils::resolve_og_image;

fn hosts() -> HostPolicy {
    HostPolicy::new(
        vec![
            "localhost:3000/".to_string(),
            "127.0.0.1:3000".to_string(),
            "www.example.com/".to_string(),
        ],
        true,
    )
}

#[test]
fn an_absolute_url_is_kept() {
    for url in ["http://cdn.example/a.png", "https://cdn.example/a.png"] {
        assert_eq!(resolve_og_image(&hosts(), false, url), url);
        assert_eq!(resolve_og_image(&hosts(), true, url), url);
    }
}

#[test]
fn debug_uses_the_local_host_over_http() {
    assert_eq!(
        resolve_og_image(&hosts(), true, "/static/og.png"),
        "http://localhost:3000/static/og.png"
    );
    let only_ip = HostPolicy::new(vec!["127.0.0.1:8000".to_string()], true);
    assert_eq!(
        resolve_og_image(&only_ip, true, "og.png"),
        "http://127.0.0.1:8000/og.png"
    );
}

#[test]
fn production_uses_the_public_host_over_https() {
    assert_eq!(
        resolve_og_image(&hosts(), false, "/static/og.png"),
        "https://www.example.com/static/og.png"
    );
    // A host naming neither localhost nor 127.0.0.1 is the public one.
    let local_only = HostPolicy::new(vec!["localhost".to_string(), "127.0.0.1".to_string()], true);
    assert_eq!(resolve_og_image(&local_only, false, "og.png"), "/og.png");
}
