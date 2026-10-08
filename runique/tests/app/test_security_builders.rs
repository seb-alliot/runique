//! What the middleware builders configure reaches the responses of a real app
//! in production mode — written from cargo-mutants survivors (2026-10-02):
//! each builder below could be replaced by its default without a test failing.
use axum::Router;
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::middleware::security::ClientIp;
use sea_orm::Database;
use serial_test::serial;
use std::net::SocketAddr;

async fn spawn_prod_app() -> String {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = false;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();

    let routes = Router::new().route("/", get(|| async { "ok" })).route(
        "/ip",
        get(|axum::Extension(ip): axum::Extension<ClientIp>| async move { ip.0.to_string() }),
    );
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .middleware(|m| {
            m.with_session_store(tower_sessions::MemoryStore::default())
                .with_csp(|c| {
                    c.objects(vec!["'none'"])
                        .frame_ancestors(vec!["https://parent.example"])
                        .base_uri(vec!["'self'"])
                })
                .with_cors(|c| c.origin("https://app.example").max_age(600))
                .with_permissions_policy(|p| p.allow_self("camera"))
                .with_trusted_proxies(|t| t.none())
        })
        .build()
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router;
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}

fn header(resp: &reqwest::Response, name: &str) -> String {
    resp.headers()
        .get_all(name)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
#[serial]
async fn configured_security_headers_reach_the_response() {
    let base = spawn_prod_app().await;
    let client = reqwest::Client::new();
    let resp = client.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let csp = header(&resp, "content-security-policy");
    assert!(csp.contains("object-src 'none'"), "{csp}");
    assert!(
        csp.contains("frame-ancestors https://parent.example"),
        "{csp}"
    );
    assert!(csp.contains("base-uri 'self'"), "{csp}");

    let pp = header(&resp, "permissions-policy");
    assert!(pp.contains("camera=(self)"), "{pp}");

    // Production: the custom store's session cookie never travels over plain HTTP.
    let cookie = header(&resp, "set-cookie");
    assert!(!cookie.is_empty(), "a session cookie is set");
    assert!(cookie.to_ascii_lowercase().contains("secure"), "{cookie}");
}

#[tokio::test]
#[serial]
async fn cors_answers_only_the_configured_origin() {
    let base = spawn_prod_app().await;
    let client = reqwest::Client::new();
    let preflight = |origin: &'static str| {
        client
            .request(reqwest::Method::OPTIONS, format!("{base}/"))
            .header("origin", origin)
            .header("access-control-request-method", "GET")
            .send()
    };
    let allowed = preflight("https://app.example").await.unwrap();
    assert_eq!(
        header(&allowed, "access-control-allow-origin"),
        "https://app.example"
    );
    assert_eq!(header(&allowed, "access-control-max-age"), "600");
    let other = preflight("https://evil.example").await.unwrap();
    assert_eq!(header(&other, "access-control-allow-origin"), "");
}

#[tokio::test]
#[serial]
async fn a_forwarded_ip_is_ignored_when_no_proxy_is_trusted() {
    let base = spawn_prod_app().await;
    let ip = reqwest::Client::new()
        .get(format!("{base}/ip"))
        .header("x-forwarded-for", "1.2.3.4")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(ip, "127.0.0.1", "the header can't spoof the client address");
}

async fn build_with_key(debug: bool, key: &str) -> Result<RuniqueApp, String> {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = debug;
    config.server.secret_key = key.into();
    RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .map_err(|e| e.to_string())
}

/// Production refuses to boot on a key that can't protect CSRF tokens and
/// sessions; development only warns.
#[tokio::test]
#[serial]
async fn production_refuses_a_weak_secret_key() {
    for weak in ["", "default_secret_key", "short-key"] {
        let err = build_with_key(false, weak).await.err().expect("refused");
        assert!(err.contains("SECRET_KEY"), "{err}");
    }
    assert!(build_with_key(true, "short-key").await.is_ok());
    assert!(
        build_with_key(
            false,
            "a-long-enough-test-secret-key-for-production-0123456789"
        )
        .await
        .is_ok()
    );
}

// ── ENFORCE_HTTPS: mounted behind a proxy, never alongside ACME ──────────────

async fn spawn_https_app(acme: bool) -> String {
    spawn_https_app_with(true, acme).await
}

async fn spawn_https_app_with(enforce_https: bool, acme: bool) -> String {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    config.security.enforce_https = enforce_https;
    config.security.acme_enabled = acme;
    config.security.acme_domain = Some("example.com".into());
    config.security.acme_email = Some("admin@example.com".into());
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/page", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router;
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}

async fn get_forwarded(base: &str, proto: &str) -> reqwest::Response {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
        .get(format!("{base}/page?x=1"))
        .header("x-forwarded-proto", proto)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn enforce_https_redirects_what_the_proxy_received_over_http() {
    let base = spawn_https_app(false).await;
    let resp = get_forwarded(&base, "http").await;
    assert_eq!(resp.status().as_u16(), 308);
    let host = base.trim_start_matches("http://");
    assert_eq!(
        header(&resp, "location"),
        format!("https://{host}/page?x=1")
    );
    assert_eq!(get_forwarded(&base, "https").await.status().as_u16(), 200);
}

// With ACME, Runique serves TLS itself: no request carries the header, and its
// port-80 listener already redirects — mounting it would only risk a loop.
#[cfg(feature = "acme")]
#[tokio::test]
#[serial]
async fn enforce_https_is_not_mounted_alongside_acme() {
    let base = spawn_https_app(true).await;
    assert_eq!(get_forwarded(&base, "http").await.status().as_u16(), 200);
}

/// Production app answering "ok" on `/`, with host validation for `good.example`
/// switched on or off.
async fn spawn_host_app(enabled: bool) -> String {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = false;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .middleware(|m| {
            m.with_session_store(tower_sessions::MemoryStore::default())
                .with_allowed_hosts(|h| h.enabled(enabled).host("good.example"))
                .with_trusted_proxies(|t| t.none())
        })
        .build()
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router;
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}

async fn as_host(base: &str, host: &str) -> (u16, String) {
    let resp = reqwest::Client::new()
        .get(format!("{base}/"))
        .header("host", host)
        .send()
        .await
        .unwrap();
    (resp.status().as_u16(), resp.text().await.unwrap())
}

/// Host validation, end to end: when on, only the listed host reaches the
/// page; when off, any host does. Written from cargo-mutants survivors
/// (2026-10-08): allowed_hosts.rs:98, 125 (allowed_hosts_middleware).
#[tokio::test]
#[serial]
async fn host_validation_lets_only_the_listed_host_through() {
    let base = spawn_host_app(true).await;
    assert_eq!(
        as_host(&base, "good.example").await,
        (200, "ok".to_string())
    );
    assert_eq!(as_host(&base, "evil.example").await.0, 400);

    let base = spawn_host_app(false).await;
    assert_eq!(
        as_host(&base, "evil.example").await,
        (200, "ok".to_string())
    );
}

// ── Written from cargo-mutants survivors (2026-10-08): applicator.rs ─────────

/// Without `ENFORCE_HTTPS`, a request the proxy received over http is served.
/// applicator.rs:220.
#[tokio::test]
#[serial]
async fn no_https_redirect_unless_enforced() {
    let base = spawn_https_app_with(false, false).await;
    assert_eq!(get_forwarded(&base, "http").await.status().as_u16(), 200);
}

/// Built without the `acme` feature, the ACME flag can't switch the redirect
/// off: Runique doesn't serve TLS itself then. applicator.rs:219.
#[cfg(not(feature = "acme"))]
#[tokio::test]
#[serial]
async fn the_acme_flag_alone_does_not_drop_the_https_redirect() {
    let base = spawn_https_app(true).await;
    assert_eq!(get_forwarded(&base, "http").await.status().as_u16(), 308);
}

/// An app in debug or production mode with the default session store, CORS
/// without a max age.
async fn spawn_mode_app(debug: bool) -> String {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = debug;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .middleware(|m| {
            m.with_cors(|c| c.origin("https://app.example").max_age(0))
                .with_trusted_proxies(|t| t.none())
        })
        .build()
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router;
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}

/// The session cookie is `Secure` in production only; local debug pages are
/// never cached; a CORS max age of 0 means the one-hour default.
/// applicator.rs:155, 271, 296.
#[tokio::test]
#[serial]
async fn debug_and_production_differ_where_they_should() {
    for debug in [true, false] {
        let base = spawn_mode_app(debug).await;
        let client = reqwest::Client::new();
        let resp = client.get(format!("{base}/")).send().await.unwrap();
        let cookie = header(&resp, "set-cookie");
        assert!(!cookie.is_empty(), "a session cookie (debug: {debug})");
        assert_eq!(cookie.contains("Secure"), !debug, "{cookie}");
        assert_eq!(
            header(&resp, "cache-control").contains("no-store"),
            debug,
            "debug: {debug}"
        );

        let preflight = client
            .request(reqwest::Method::OPTIONS, format!("{base}/"))
            .header("origin", "https://app.example")
            .header("access-control-request-method", "GET")
            .send()
            .await
            .unwrap();
        assert_eq!(header(&preflight, "access-control-max-age"), "3600");
    }
}
