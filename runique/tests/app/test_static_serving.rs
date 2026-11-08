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
    build_in(true, statics).await
}

async fn build_in(
    dev: bool,
    statics: impl FnOnce(runique::app::staging::StaticStaging) -> runique::app::staging::StaticStaging,
) -> (Result<Router, String>, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("rq_static_{}", uuid::Uuid::new_v4()));
    let (static_dir, media_dir) = (root.join("static"), root.join("media"));
    std::fs::create_dir_all(&static_dir).unwrap();
    std::fs::create_dir_all(&media_dir).unwrap();
    std::fs::write(static_dir.join("app.css"), "body{}").unwrap();
    std::fs::write(media_dir.join("photo.txt"), "x").unwrap();

    let mut config = RuniqueConfig::from_env();
    config.dev = dev;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
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
    let (app, root) = build_in(false, |s| {
        s.static_cache("public, max-age=60").media_cache("no-cache")
    })
    .await;
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
        "script-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A built app whose only route reads the form body through `Request`, its
/// body limit set as RUNIQUE_MAX_UPLOAD_MB would.
/// While a stylesheet is edited its URL doesn't change: the browser must ask
/// again, whatever the builder says. Media files keep their own setting.
#[tokio::test]
#[serial]
async fn development_static_files_are_never_kept_by_the_browser() {
    let (app, root) = build(|s| s.static_cache("public, max-age=60").media_cache("private")).await;
    let app = app.unwrap();
    let css = fetch(app.clone(), "/static/app.css").await;
    assert_eq!(header(&css, "cache-control"), "no-cache");
    let media = fetch(app, "/media/photo.txt").await;
    assert_eq!(header(&media, "cache-control"), "private");
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
#[serial]
async fn production_static_files_keep_the_long_cache_by_default() {
    let (app, root) = build_in(false, |s| s).await;
    let css = fetch(app.unwrap(), "/static/app.css").await;
    assert_eq!(
        header(&css, "cache-control"),
        "public, max-age=31536000, immutable"
    );
    let _ = std::fs::remove_dir_all(root);
}

async fn build_with_body_limit(max_upload_mb: u64, max_body_mb: u64) -> Router {
    // GET too: `app_with_session` reads the token a first page hands out.
    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    config.static_files.max_upload_mb = max_upload_mb;
    config.static_files.max_body_mb = max_body_mb;
    // Uploads accepted by these tests are staged under the test run's
    // MEDIA_ROOT (`target/test-media`), not in the repository; the staging
    // folder goes at the end of each request.
    config.static_files.media_root = runique::config::static_files::resolve_media_root();
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

/// A built app whose `/form` route reads the body through `Request`, and the
/// session cookie + masked CSRF token a first GET hands out.
async fn app_with_session() -> (Router, String, String) {
    app_with_session_limits(10, 10).await
}

/// [`app_with_session`] with its own per-file and request limits (MB).
async fn app_with_session_limits(max_upload_mb: u64, max_body_mb: u64) -> (Router, String, String) {
    let app = build_with_body_limit(max_upload_mb, max_body_mb).await;
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

// A body read into memory (urlencoded) stays at 2 MB, whatever the multipart
// limit: 100 MB of RAM per request would be open to any visitor.
#[tokio::test]
#[serial]
async fn a_form_body_stays_at_two_megabytes() {
    const MB: usize = 1024 * 1024;
    let app = build_with_body_limit(100, 100).await;
    assert_eq!(
        post_form(app.clone(), 2 * MB - 1024).await,
        StatusCode::OK,
        "just under"
    );
    assert_eq!(
        post_form(app, 2 * MB + 1024).await,
        StatusCode::PAYLOAD_TOO_LARGE,
        "just over"
    );
}

/// One multipart POST to `/form`: the CSRF token, then one PDF per entry of
/// `sizes` (exact sizes in bytes). `announced`, when given, is sent as the
/// request's `Content-Length` whatever the body really weighs.
async fn post_files(
    app: Router,
    cookie: &str,
    token: &str,
    sizes: &[usize],
    announced: Option<u64>,
) -> axum::response::Response {
    let b = "BNDRY";
    let mut body =
        format!("--{b}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{token}\r\n");
    for (i, size) in sizes.iter().enumerate() {
        body.push_str(&format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"doc{i}\"; filename=\"{i}.pdf\"\r\n\r\n%PDF{}\r\n",
            "a".repeat(size - 4)
        ));
    }
    body.push_str(&format!("--{b}--\r\n"));
    let mut req = Request::builder()
        .method("POST")
        .uri("/form")
        .header("cookie", cookie)
        .header("content-type", format!("multipart/form-data; boundary={b}"));
    if let Some(len) = announced {
        req = req.header("content-length", len);
    }
    app.oneshot(req.body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

const KB: usize = 1024;
const MB_BYTES: usize = 1024 * 1024;

// Each file is held to the per-file limit, one byte included.
#[tokio::test]
#[serial]
async fn a_file_is_held_to_the_per_file_limit() {
    let (app, cookie, token) = app_with_session_limits(1, 5).await;
    assert_eq!(
        post_files(app.clone(), &cookie, &token, &[MB_BYTES], None)
            .await
            .status(),
        StatusCode::OK,
        "exactly 1 MB"
    );
    assert_eq!(
        post_files(app, &cookie, &token, &[MB_BYTES + 1], None)
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "one byte more"
    );
}

// Several files each under the per-file limit pass while their sum fits the
// request limit, and are refused past it.
#[tokio::test]
#[serial]
async fn the_request_limit_holds_the_sum_of_the_files() {
    // 1 MB per file, 2 MB (+ 1 MB of text fields) per request.
    let (app, cookie, token) = app_with_session_limits(1, 2).await;
    assert_eq!(
        post_files(app.clone(), &cookie, &token, &[900 * KB; 3], None)
            .await
            .status(),
        StatusCode::OK,
        "3 x 900 KB: 2.7 MB, under 3 MB"
    );
    assert_eq!(
        post_files(app, &cookie, &token, &[900 * KB; 4], None)
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "4 x 900 KB: 3.6 MB, over 3 MB"
    );
}

// A request announcing a size over the limit is refused on its headers: the
// tiny body it really carries proves nothing was read to decide.
#[tokio::test]
#[serial]
async fn a_request_announced_over_the_limit_is_refused_before_reading() {
    let (app, cookie, token) = app_with_session_limits(1, 1).await;
    let limit = 2 * 1024 * 1024; // 1 MB + 1 MB of text fields
    assert_eq!(
        post_files(app.clone(), &cookie, &token, &[10], Some(limit))
            .await
            .status(),
        StatusCode::OK,
        "announced exactly at the limit"
    );
    assert_eq!(
        post_files(app, &cookie, &token, &[10], Some(limit + 1))
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "announced one byte over"
    );
}

// Runique's own 413 is the `413.html` page, with the reason in the visitor's
// language, not a bare text.
#[tokio::test]
#[serial]
async fn a_refused_upload_shows_the_413_page_with_its_reason() {
    use http_body_util::BodyExt;
    let (app, cookie, token) = app_with_session_limits(1, 1).await;
    let resp = post_files(app, &cookie, &token, &[10], Some(10 * 1024 * 1024)).await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(
        header(&resp, "content-type").starts_with("text/html"),
        "a page, not a bare text"
    );
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("<h1"), "{html}");
    assert!(
        html.contains(runique::utils::trad::t("forms.body_too_large").as_ref()),
        "the reason of this 413: {html}"
    );
}
