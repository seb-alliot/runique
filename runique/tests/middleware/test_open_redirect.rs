use crate::helpers::{
    assert::{assert_redirect, assert_status},
    request,
    server::build_engine,
};
use axum::{Router, middleware, response::Redirect, routing::get};
use runique::{
    engine::RuniqueEngine,
    middleware::{
        config::MiddlewareConfig,
        security::{
            allowed_hosts::HostPolicy, csp::SecurityPolicy, open_redirect::open_redirect_middleware,
        },
    },
};
use std::sync::Arc;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn redirect_app(engine: Arc<RuniqueEngine>, location: &'static str) -> Router {
    Router::new()
        .route("/", get(move || async move { Redirect::to(location) }))
        .layer(middleware::from_fn_with_state(
            engine,
            open_redirect_middleware,
        ))
}

async fn engine_with_hosts(allowed: Vec<&str>) -> Arc<RuniqueEngine> {
    let hosts: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let base = build_engine().await;
    Arc::new(RuniqueEngine {
        config: base.config.clone(),
        tera: base.tera.clone(),
        db: base.db.clone(),
        url_registry: base.url_registry.clone(),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(hosts, true)),
        csrf_exempt_paths: Arc::new(vec![]),
        permissions_policy: Arc::new(runique::middleware::PermissionsPolicy::default()),
        trusted_proxies: Arc::new(runique::middleware::TrustedProxies::default()),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        custom_db: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
    })
}

// ── Fonctions unitaires ───────────────────────────────────────────────────────

#[cfg(test)]
mod unit {
    use runique::middleware::security::open_redirect::{extract_host, is_local_host};

    #[test]
    fn extract_host_https() {
        assert_eq!(extract_host("https://evil.com/path"), Some("evil.com"));
    }

    #[test]
    fn extract_host_http_with_port() {
        assert_eq!(
            extract_host("http://evil.com:8080/path"),
            Some("evil.com:8080")
        );
    }

    #[test]
    fn extract_host_protocol_relative() {
        assert_eq!(extract_host("//evil.com/path"), Some("evil.com"));
    }

    #[test]
    fn extract_host_relative_returns_none() {
        assert_eq!(extract_host("/relative/path"), None);
    }

    #[test]
    fn extract_host_no_path() {
        assert_eq!(extract_host("https://evil.com"), Some("evil.com"));
    }

    #[test]
    fn extract_host_query_string() {
        assert_eq!(extract_host("https://evil.com?q=1"), Some("evil.com"));
    }

    #[test]
    fn extract_host_fragment() {
        assert_eq!(extract_host("https://evil.com#section"), Some("evil.com"));
    }

    #[test]
    fn is_local_host_localhost() {
        assert!(is_local_host("localhost"));
    }

    #[test]
    fn is_local_host_localhost_with_port() {
        assert!(is_local_host("localhost:8080"));
    }

    #[test]
    fn is_local_host_127() {
        assert!(is_local_host("127.0.0.1"));
    }

    #[test]
    fn is_local_host_127_with_port() {
        assert!(is_local_host("127.0.0.1:3000"));
    }

    #[test]
    fn is_local_host_ipv6() {
        assert!(is_local_host("[::1]"));
    }

    #[test]
    fn is_local_host_ipv4_loopback_range() {
        // Toute la plage 127.0.0.0/8 est loopback
        assert!(is_local_host("127.0.0.2"));
        assert!(is_local_host("127.1.2.3"));
        assert!(is_local_host("127.255.255.255"));
        assert!(is_local_host("127.0.0.1:8080"));
    }

    #[test]
    fn is_local_host_ipv6_full_form() {
        assert!(is_local_host("[0:0:0:0:0:0:0:1]"));
        assert!(is_local_host("[0:0:0:0:0:0:0:1]:8080"));
    }

    #[test]
    fn is_local_host_ipv6_mapped_ipv4() {
        assert!(is_local_host("[::ffff:127.0.0.1]"));
        assert!(is_local_host("[::ffff:7f00:1]"));
    }

    #[test]
    fn is_local_host_external() {
        assert!(!is_local_host("evil.com"));
        assert!(!is_local_host("128.0.0.1")); // pas loopback
    }

    #[test]
    fn is_local_host_looks_like_localhost() {
        assert!(!is_local_host("notlocalhost.com"));
        assert!(!is_local_host("localhost.evil.com"));
    }
}

// ── Intégration — redirections laissées passer ────────────────────────────────

#[tokio::test]
async fn relative_redirect_passes() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "/dashboard"), "/").await;
    assert_redirect(&resp, "/dashboard");
}

#[tokio::test]
async fn relative_redirect_with_query_passes() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "/login?next=/home"), "/").await;
    assert_redirect(&resp, "/login?next=/home");
}

#[tokio::test]
async fn localhost_absolute_redirect_passes() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "http://localhost:8080/path"), "/").await;
    assert_redirect(&resp, "http://localhost:8080/path");
}

#[tokio::test]
async fn localhost_127_redirect_passes() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "http://127.0.0.1:3000/cb"), "/").await;
    assert_redirect(&resp, "http://127.0.0.1:3000/cb");
}

#[tokio::test]
async fn non_redirect_response_passes_unchanged() {
    let engine = build_engine().await;
    let app =
        Router::new()
            .route("/", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                engine,
                open_redirect_middleware,
            ));
    let resp = request::get(app, "/").await;
    assert_status(&resp, 200);
}

#[tokio::test]
async fn allowed_host_redirect_passes() {
    let engine = engine_with_hosts(vec!["myapp.com"]).await;
    let resp = request::get(redirect_app(engine, "https://myapp.com/callback"), "/").await;
    assert_redirect(&resp, "https://myapp.com/callback");
}

#[tokio::test]
async fn allowed_wildcard_subdomain_passes() {
    let engine = engine_with_hosts(vec![".myapp.com"]).await;
    let resp = request::get(redirect_app(engine, "https://auth.myapp.com/oauth"), "/").await;
    assert_redirect(&resp, "https://auth.myapp.com/oauth");
}

// ── Intégration — redirections bloquées ──────────────────────────────────────

#[tokio::test]
async fn external_absolute_redirect_blocked() {
    let engine = build_engine().await; // aucun host autorisé
    let resp = request::get(redirect_app(engine, "https://evil.com/steal"), "/").await;
    assert_status(&resp, 400);
}

#[tokio::test]
async fn protocol_relative_redirect_blocked() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "//evil.com/steal"), "/").await;
    assert_status(&resp, 400);
}

#[tokio::test]
async fn external_http_redirect_blocked() {
    let engine = build_engine().await;
    let resp = request::get(redirect_app(engine, "http://evil.com/phishing"), "/").await;
    assert_status(&resp, 400);
}

#[tokio::test]
async fn not_in_allowed_hosts_blocked() {
    let engine = engine_with_hosts(vec!["myapp.com"]).await;
    let resp = request::get(redirect_app(engine, "https://other.com/path"), "/").await;
    assert_status(&resp, 400);
}

#[tokio::test]
async fn subdomain_spoof_blocked() {
    // "myapp.com.evil.com" ne doit pas passer avec allowed ".myapp.com"
    let engine = engine_with_hosts(vec![".myapp.com"]).await;
    let resp = request::get(
        redirect_app(engine, "https://myapp.com.evil.com/steal"),
        "/",
    )
    .await;
    assert_status(&resp, 400);
}

#[tokio::test]
async fn lookalike_host_blocked() {
    // "notmyapp.com" ne doit pas passer avec "myapp.com"
    let engine = engine_with_hosts(vec!["myapp.com"]).await;
    let resp = request::get(redirect_app(engine, "https://notmyapp.com/path"), "/").await;
    assert_status(&resp, 400);
}
