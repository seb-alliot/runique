//! The debug error page lists the request's headers, minus every sensitive one
//! (credentials, cookies, sessions, tokens, keys, secrets, signatures): such a
//! page ends up in screenshots, shared screens and issues. Checked on the real
//! page, rendered by a real app with the framework's templates.

use axum::{Router, http::StatusCode, routing::get};
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;

/// Headers whose value must never reach the page: the usual names for
/// credentials, sessions, keys, secrets and signatures, in their common
/// spellings. One distinct value each, so a leak names its header.
const SECRET_HEADERS: &[(&str, &str)] = &[
    // Credentials
    ("Authorization", "Bearer SECRET-01"),
    ("Proxy-Authorization", "Basic SECRET-02"),
    ("X-Forwarded-Authorization", "Bearer SECRET-03"),
    ("WWW-Authenticate", "SECRET-04"),
    ("X-Auth", "SECRET-05"),
    ("X-Auth-User", "SECRET-06"),
    ("X-Password", "SECRET-07"),
    ("X-Passwd", "SECRET-08"),
    ("X-Credentials", "SECRET-09"),
    // Cookies and sessions
    ("Cookie", "sessionid=SECRET-10"),
    ("Set-Cookie", "SECRET-11"),
    ("X-Session-Id", "SECRET-12"),
    ("X-Session", "SECRET-13"),
    // Tokens
    ("X-CSRF-Token", "SECRET-14"),
    ("X-CSRFToken", "SECRET-15"),
    ("X-XSRF-TOKEN", "SECRET-16"),
    ("X-Auth-Token", "SECRET-17"),
    ("X-Access-Token", "SECRET-18"),
    ("X-Refresh-Token", "SECRET-19"),
    ("X-Amz-Security-Token", "SECRET-20"),
    ("Private-Token", "SECRET-21"),
    // Keys
    ("X-Api-Key", "SECRET-22"),
    ("Api-Key", "SECRET-23"),
    ("X-Goog-Api-Key", "SECRET-24"),
    ("Ocp-Apim-Subscription-Key", "SECRET-25"),
    ("Sec-WebSocket-Key", "SECRET-26"),
    // Secrets and signatures
    ("X-Client-Secret", "SECRET-27"),
    ("X-Webhook-Secret", "SECRET-28"),
    ("X-Hub-Signature-256", "SECRET-29"),
    ("X-Signature", "SECRET-30"),
];

/// An ordinary header, which the page does show: proves headers are rendered
/// at all, so a missing secret means filtered, not absent.
const VISIBLE_HEADER: (&str, &str) = ("X-Trace-Id", "TRACE-VISIBLE");

async fn spawn(debug: bool) -> String {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let mut config = RuniqueConfig::from_env();
    config.debug = debug;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/boom", get(|| async { StatusCode::INTERNAL_SERVER_ERROR })))
        .static_files(|s| s.enabled(false))
        .middleware(|m| m.with_session_store(tower_sessions::MemoryStore::default()))
        .build()
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.router
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{addr}")
}

/// GETs `path` carrying every secret header and the visible one.
async fn error_page(base: &str, path: &str) -> (u16, String) {
    let mut req = reqwest::Client::new().get(format!("{base}{path}"));
    for (name, value) in SECRET_HEADERS.iter().chain([&VISIBLE_HEADER]) {
        req = req.header(*name, *value);
    }
    let resp = req.send().await.unwrap();
    (resp.status().as_u16(), resp.text().await.unwrap())
}

fn assert_no_secret(html: &str) {
    let shown: Vec<&str> = SECRET_HEADERS
        .iter()
        .filter(|(_, value)| html.contains(value))
        .map(|(name, _)| *name)
        .collect();
    assert!(shown.is_empty(), "shown on the page: {shown:?}");
}

#[tokio::test]
#[serial]
async fn the_debug_page_hides_every_sensitive_header_on_a_500() {
    let base = spawn(true).await;
    let (status, html) = error_page(&base, "/boom").await;
    assert_eq!(status, 500);
    assert!(
        html.contains(VISIBLE_HEADER.1),
        "headers are rendered on the debug page"
    );
    assert_no_secret(&html);
}

#[tokio::test]
#[serial]
async fn the_debug_page_hides_every_sensitive_header_on_a_404() {
    let base = spawn(true).await;
    let (status, html) = error_page(&base, "/nowhere").await;
    assert_eq!(status, 404);
    assert!(
        html.contains(VISIBLE_HEADER.1),
        "headers are rendered on the debug page"
    );
    assert_no_secret(&html);
}

// Outside debug, the error page shows no request detail at all.
#[tokio::test]
#[serial]
async fn the_production_error_page_shows_no_header() {
    let base = spawn(false).await;
    for path in ["/boom", "/nowhere"] {
        let (_, html) = error_page(&base, path).await;
        assert!(!html.contains(VISIBLE_HEADER.1), "{path}");
        assert_no_secret(&html);
    }
}
