//! Static and media files served by a built app, with their headers — written
//! from cargo-mutants survivors (2026-10-02): `attach_static_files` and the
//! cache builders could be dropped without a test failing.
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;
use tower::ServiceExt;

async fn build(
    statics: impl FnOnce(runique::app::staging::StaticStaging) -> runique::app::staging::StaticStaging,
) -> (Result<Router, String>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("rq_static_{}", uuid::Uuid::new_v4()));
    let (static_dir, media_dir) = (root.join("static"), root.join("media"));
    std::fs::create_dir_all(&static_dir).unwrap();
    std::fs::create_dir_all(&media_dir).unwrap();
    std::fs::write(static_dir.join("app.css"), "body{}").unwrap();
    std::fs::write(media_dir.join("photo.txt"), "x").unwrap();

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    config.static_files.staticfiles_dirs = static_dir.to_string_lossy().into_owned();
    config.static_files.media_root = media_dir.to_string_lossy().into_owned();
    config.static_files.static_url = "/static".into();
    config.static_files.media_url = "/media".into();

    let app = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(statics)
        .build()
        .await
        .map(|a| a.router)
        .map_err(|e| e.to_string());
    (app, root)
}

async fn fetch(app: Router, uri: &str) -> axum::response::Response {
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

fn header<'a>(resp: &'a axum::response::Response, name: &str) -> &'a str {
    resp.headers().get(name).map_or("", |v| v.to_str().unwrap())
}

#[tokio::test]
#[serial]
async fn static_and_media_files_carry_their_cache_and_security_headers() {
    let (app, root) = build(|s| s.static_cache("public, max-age=60").media_cache("no-cache")).await;
    let app = app.unwrap();

    let css = fetch(app.clone(), "/static/app.css").await;
    assert_eq!(css.status(), StatusCode::OK);
    assert_eq!(header(&css, "cache-control"), "public, max-age=60");
    assert_eq!(header(&css, "x-content-type-options"), "nosniff");
    assert_eq!(header(&css, "x-frame-options"), "DENY");

    let media = fetch(app, "/media/photo.txt").await;
    assert_eq!(media.status(), StatusCode::OK);
    assert_eq!(header(&media, "cache-control"), "no-cache");
    assert_eq!(header(&media, "x-content-type-options"), "nosniff");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
#[serial]
async fn an_invalid_cache_header_is_a_build_error_not_a_panic() {
    let (app, root) = build(|s| s.static_cache("public\nmax-age=60")).await;
    let err = app.expect_err("refused");
    assert!(err.contains("static_cache"), "{err}");
    let (app, root2) = build(|s| s.enabled(false).static_cache("bad\nvalue")).await;
    assert!(app.is_ok(), "not checked when static files are off");
    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::remove_dir_all(root2);
}

// Uploads wait in `.staging-*` folders under the media root until their form
// is checked: never reachable, even percent-encoded (`%2E` is a dot to ServeDir).
#[tokio::test]
#[serial]
async fn a_dot_folder_under_media_is_never_served() {
    let (app, root) = build(|s| s).await;
    let app = app.unwrap();
    let staging = root.join("media").join(".staging-x");
    std::fs::create_dir_all(&staging).unwrap();
    std::fs::write(staging.join("page.html"), "<script>").unwrap();

    for uri in [
        "/media/.staging-x/page.html",
        "/media/%2Estaging-x/page.html",
        "/media/%2estaging-x/page.html",
    ] {
        assert_eq!(
            fetch(app.clone(), uri).await.status(),
            StatusCode::NOT_FOUND,
            "{uri}"
        );
    }
    assert_eq!(
        fetch(app, "/media/photo.txt").await.status(),
        StatusCode::OK,
        "a plain file still is"
    );
    let _ = std::fs::remove_dir_all(root);
}

// An uploaded `.html` is served from the site's own origin: it must not run.
#[tokio::test]
#[serial]
async fn media_responses_forbid_scripts() {
    let (app, root) = build(|s| s).await;
    let media = fetch(app.unwrap(), "/media/photo.txt").await;
    assert_eq!(
        header(&media, "content-security-policy"),
        "script-src 'none'; object-src 'none'; base-uri 'none'"
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A built app whose only route reads the form body through `Request`, its
/// body limit set as RUNIQUE_MAX_UPLOAD_MB would.
async fn build_with_upload_limit(max_upload_mb: u64) -> Router {
    build_with_body_limit(max_upload_mb, Some(max_upload_mb)).await
}

async fn build_with_body_limit(max_upload_mb: u64, max_body_mb: Option<u64>) -> Router {
    // GET too: `app_with_session` reads the token a first page hands out.
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    config.static_files.max_upload_mb = max_upload_mb;
    config.static_files.max_body_mb = max_body_mb;
    // Uploads accepted by these tests are staged here, not in the repository.
    config.static_files.media_root = std::env::temp_dir()
        .join(format!("rq_upload_{}", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .into_owned();
    async fn read_form(_request: runique::prelude::Request) -> &'static str {
        "read"
    }
    RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new().route("/form", axum::routing::get(read_form).post(read_form)))
        .build()
        .await
        .unwrap()
        .router
}

async fn post_form(app: Router, bytes: usize) -> StatusCode {
    let body = format!("title={}", "a".repeat(bytes));
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/form")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .unwrap(),
    )
    .await
    .unwrap()
    .status()
}

// The body limit is RUNIQUE_MAX_UPLOAD_MB plus 1 MB for the text fields —
// axum alone stops at 2 MB, and a raw `collect()` had no limit at all.
#[tokio::test]
#[serial]
async fn the_body_limit_follows_the_upload_setting() {
    const MB: usize = 1024 * 1024;
    let three_mb = 3 * MB;
    assert_eq!(
        post_form(build_with_upload_limit(4).await, three_mb).await,
        StatusCode::OK,
        "under 5 MB"
    );
    assert_ne!(
        post_form(build_with_upload_limit(1).await, three_mb).await,
        StatusCode::OK,
        "over 2 MB"
    );
    assert_eq!(
        post_form(build_with_upload_limit(1).await, 2 * MB - 1024).await,
        StatusCode::OK,
        "just under the limit"
    );
}

/// A built app whose `/form` route reads the body through `Request`, and the
/// session cookie + masked CSRF token a first GET hands out.
async fn app_with_session() -> (Router, String, String) {
    let app = build_with_upload_limit(10).await;
    let resp = fetch(app.clone(), "/form").await;
    let cookie = resp
        .headers()
        .get("set-cookie")
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_string())
        .expect("session cookie");
    let token = header(&resp, "x-csrf-token").to_string();
    assert!(!token.is_empty(), "token handed out");
    (app, cookie, token)
}

async fn post_upload(
    app: Router,
    cookie: &str,
    header_token: Option<&str>,
    body_token: &str,
) -> StatusCode {
    let b = "BNDRY";
    // The file comes first: a body token can't vouch for it.
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"doc\"; filename=\"a.pdf\"\r\n\r\n%PDF\r\n\
         --{b}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{body_token}\r\n--{b}--\r\n"
    );
    let mut req = Request::builder()
        .method("POST")
        .uri("/form")
        .header("cookie", cookie)
        .header("content-type", format!("multipart/form-data; boundary={b}"));
    if let Some(token) = header_token {
        req = req.header("x-csrf-token", token);
    }
    app.oneshot(req.body(Body::from(body)).unwrap())
        .await
        .unwrap()
        .status()
}

// The request pipeline turns the upload gate on unless the header carries the
// session's token: a file before the body token is refused, not written.
#[tokio::test]
#[serial]
async fn an_upload_needs_the_token_before_its_file_unless_the_header_has_it() {
    let (app, cookie, token) = app_with_session().await;
    assert_eq!(
        post_upload(app.clone(), &cookie, None, &token).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        post_upload(app, &cookie, Some(&token), &token).await,
        StatusCode::OK
    );
}

// A multipart body over the limit is refused, not read up to the limit and
// handed to the form cut short.
#[tokio::test]
#[serial]
async fn a_multipart_body_over_the_limit_is_refused_not_cut_short() {
    let (app, cookie, token) = app_with_session().await;
    let b = "BNDRY";
    // 200 fields of 60 KB: each under the text-field limit, 12 MB in all
    // against the 11 MB body limit.
    let chunk = "a".repeat(60 * 1024);
    let mut body =
        format!("--{b}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{token}\r\n");
    for i in 0..200 {
        body.push_str(&format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"f{i}\"\r\n\r\n{chunk}\r\n"
        ));
    }
    body.push_str(&format!("--{b}--\r\n"));
    let req = Request::builder()
        .method("POST")
        .uri("/form")
        .header("cookie", cookie)
        .header("content-type", format!("multipart/form-data; boundary={b}"))
        .body(Body::from(body))
        .unwrap();
    assert_eq!(
        app.oneshot(req).await.unwrap().status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

// RUNIQUE_MAX_UPLOAD_MB unset: axum's own 2 MB stay the limit.
#[tokio::test]
#[serial]
async fn without_the_setting_axum_keeps_its_two_megabytes() {
    const MB: usize = 1024 * 1024;
    let app = build_with_body_limit(100, None).await;
    assert_eq!(
        post_form(app.clone(), MB + MB / 2).await,
        StatusCode::OK,
        "1.5 MB"
    );
    assert_ne!(post_form(app, 3 * MB).await, StatusCode::OK, "3 MB");
}
