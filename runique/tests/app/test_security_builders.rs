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
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    config.security.enforce_https = true;
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
