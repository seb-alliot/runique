//! `.with_lang()`: each visitor is answered in the language its browser asks
//! for, for the time of its request, and the answer says it depends on it.
//! Without a language the application has, and without the middleware, the
//! application's own language stays.
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Request, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use http_body_util::BodyExt;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::middleware::lang_middleware;
use runique::translation::{Lang, current_lang};
use runique_i18n::set_lang;
use sea_orm::Database;
use serial_test::serial;
use tower::ServiceExt;

async fn spoken() -> &'static str {
    // Across a pause: the language follows the request, not the thread.
    tokio::task::yield_now().await;
    current_lang().code()
}

async fn already_varying() -> Response {
    ([(header::VARY, "accept-encoding")], "ok").into_response()
}

fn app() -> Router {
    Router::new()
        .route("/", get(spoken))
        .route("/varying", get(already_varying))
        .layer(middleware::from_fn(lang_middleware))
}

fn request(uri: &str, accept_language: Option<HeaderValue>) -> Request<Body> {
    let mut req = Request::builder().uri(uri);
    if let Some(value) = accept_language {
        req = req.header(header::ACCEPT_LANGUAGE, value);
    }
    req.body(Body::empty()).unwrap()
}

fn asking(value: &'static str) -> Option<HeaderValue> {
    Some(HeaderValue::from_static(value))
}

async fn text(resp: Response) -> String {
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&body).into_owned()
}

fn vary(resp: &Response) -> Vec<String> {
    resp.headers()
        .get_all(header::VARY)
        .iter()
        .map(|v| v.to_str().unwrap().to_lowercase())
        .collect()
}

/// The application's language set to `lang` for a test, the former one back after.
struct Site(Lang);

impl Site {
    fn in_lang(lang: Lang) -> Self {
        let before = current_lang();
        set_lang(lang);
        Self(before)
    }
}

impl Drop for Site {
    fn drop(&mut self) {
        set_lang(self.0);
    }
}

#[tokio::test]
#[serial]
async fn a_visitor_is_answered_in_the_language_of_its_browser() {
    let _site = Site::in_lang(Lang::En);
    for (header, code) in [
        ("fr", "fr"),
        ("de-AT, fr;q=0.8", "de"),
        ("sv, it;q=0.8, en;q=0.5", "it"),
        ("en;q=0.2, ja;q=0.9", "ja"),
    ] {
        let resp = app().oneshot(request("/", asking(header))).await.unwrap();
        assert_eq!(text(resp).await, code, "{header}");
    }
    assert_eq!(current_lang(), Lang::En, "the application keeps its own");
}

#[tokio::test]
#[serial]
async fn without_a_language_the_application_has_its_own_stays() {
    let _site = Site::in_lang(Lang::De);
    let unreadable = HeaderValue::from_bytes(b"fr\xff").unwrap();
    for header in [
        None,
        asking("sv, da"),
        asking("fr;q=0"),
        asking(""),
        Some(unreadable),
    ] {
        let shown = format!("{header:?}");
        let resp = app().oneshot(request("/", header)).await.unwrap();
        assert_eq!(text(resp).await, "de", "{shown}");
    }
}

// Two visitors at the same time: neither sees the other's language.
#[tokio::test]
#[serial]
async fn two_requests_at_once_each_keep_their_language() {
    let _site = Site::in_lang(Lang::En);
    let app = app();
    let (french, german, none) = tokio::join!(
        app.clone().oneshot(request("/", asking("fr"))),
        app.clone().oneshot(request("/", asking("de"))),
        app.clone().oneshot(request("/", None)),
    );
    assert_eq!(text(french.unwrap()).await, "fr");
    assert_eq!(text(german.unwrap()).await, "de");
    assert_eq!(text(none.unwrap()).await, "en");
}

// Every answer depends on the header, the ones in the application's language
// too; a `Vary` the page already has is kept.
#[tokio::test]
#[serial]
async fn every_answer_says_it_varies_with_the_language_asked() {
    for header in [asking("fr"), asking("sv"), None] {
        let resp = app().oneshot(request("/", header)).await.unwrap();
        assert_eq!(vary(&resp), ["accept-language"]);
    }
    let resp = app()
        .oneshot(request("/varying", asking("fr")))
        .await
        .unwrap();
    assert_eq!(vary(&resp), ["accept-encoding", "accept-language"]);
}

// What the visitor sends is never written back.
#[tokio::test]
#[serial]
async fn the_header_is_never_echoed() {
    let sent = "zz-<script>alert(1)</script>, fr";
    let resp = app().oneshot(request("/", asking(sent))).await.unwrap();
    let headers = format!("{:?}", resp.headers());
    let body = text(resp).await;
    assert_eq!(body, "fr");
    assert!(!headers.contains("script"), "{headers}");
}

async fn built(with_lang: bool) -> Router {
    let mut config = RuniqueConfig::from_env();
    config.dev = false;
    config.debug = false;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new().route("/spoken", get(spoken)))
        .static_files(|s| s.enabled(false))
        .middleware(move |m| {
            let m = m.with_session_store(tower_sessions::MemoryStore::default());
            if with_lang { m.with_lang() } else { m }
        })
        .build()
        .await
        .expect("the app builds")
        .router
}

// Through the builder: the handler and the error page are both in the
// visitor's language, and neither is without `.with_lang()`.
#[tokio::test]
#[serial]
async fn the_builder_puts_handlers_and_error_pages_in_the_visitors_language() {
    let _site = Site::in_lang(Lang::En);
    let german_404 = Lang::De.get(runique::translation::keys::http::error::CODE_404);
    assert_ne!(
        german_404,
        Lang::En.get(runique::translation::keys::http::error::CODE_404)
    );

    let app = built(true).await;
    let resp = app
        .clone()
        .oneshot(request("/spoken", asking("de")))
        .await
        .unwrap();
    assert!(vary(&resp).contains(&"accept-language".to_string()));
    assert_eq!(text(resp).await, "de");
    let resp = app
        .clone()
        .oneshot(request("/missing", asking("de")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let page = text(resp).await;
    assert!(page.contains(&*german_404), "{page}");
    assert!(page.contains(r#"<html lang="de">"#), "{page}");
    let resp = app.oneshot(request("/spoken", asking("sv"))).await.unwrap();
    assert_eq!(text(resp).await, "en");

    let app = built(false).await;
    let resp = app
        .clone()
        .oneshot(request("/spoken", asking("de")))
        .await
        .unwrap();
    assert!(!vary(&resp).contains(&"accept-language".to_string()));
    assert_eq!(text(resp).await, "en");
    let resp = app
        .oneshot(request("/missing", asking("de")))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let page = text(resp).await;
    assert!(!page.contains(&*german_404), "{page}");
}

// ── `.with_lang("fr")` on the builder: the language of the site ──────────────

/// An application whose own language is `lang` when given, following the
/// visitor's browser or not; the refusal when it doesn't build.
async fn site(lang: Option<&str>, follow_browser: bool) -> Result<Router, String> {
    let mut config = RuniqueConfig::from_env();
    config.dev = false;
    config.debug = false;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    let mut builder = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new().route("/spoken", get(spoken)))
        .static_files(|s| s.enabled(false))
        .middleware(move |m| {
            let m = m.with_session_store(tower_sessions::MemoryStore::default());
            if follow_browser { m.with_lang() } else { m }
        });
    if let Some(code) = lang {
        builder = builder.with_lang(code);
    }
    builder
        .build()
        .await
        .map(|app| app.router)
        .map_err(|e| e.to_string())
}

async fn spoken_by(app: Router, accept_language: Option<HeaderValue>) -> String {
    let resp = app
        .oneshot(request("/spoken", accept_language))
        .await
        .unwrap();
    text(resp).await
}

// The code given to the builder is the language of the site, however it is
// written; without it the site keeps the language it had.
#[tokio::test]
#[serial]
async fn the_builder_sets_the_language_of_the_site() {
    for written in ["fr", "FR", "fr-FR", "fr_FR.UTF-8", "fr-LU"] {
        let _site = Site::in_lang(Lang::En);
        let app = site(Some(written), false).await.expect(written);
        assert_eq!(spoken_by(app, None).await, "fr", "{written}");
    }

    let _site = Site::in_lang(Lang::De);
    let app = site(None, false).await.expect("no language asked");
    assert_eq!(spoken_by(app, None).await, "de");
}

// A code no loaded language answers to stops the start, naming the code and
// the languages there are; the language of the site is left as it was.
#[tokio::test]
#[serial]
async fn a_language_nobody_has_stops_the_start() {
    let _site = Site::in_lang(Lang::It);
    for unknown in ["xx", "sv-SE", ""] {
        let refusal = site(Some(unknown), false).await.expect_err(unknown);
        assert!(refusal.contains(&format!("\"{unknown}\"")), "{refusal}");
        assert!(refusal.contains("fr, en"), "{refusal}");
        assert_eq!(current_lang(), Lang::It, "{unknown}");
    }
}

// The site's language is what a visitor gets when the browser asks for none
// the site has; following the browser stays a separate choice.
#[tokio::test]
#[serial]
async fn the_sites_language_is_the_fallback_of_the_browsers() {
    let _site = Site::in_lang(Lang::En);

    let followed = site(Some("fr"), true).await.unwrap();
    assert_eq!(spoken_by(followed.clone(), asking("de")).await, "de");
    assert_eq!(spoken_by(followed.clone(), asking("sv, da")).await, "fr");
    assert_eq!(spoken_by(followed, None).await, "fr");

    let fixed = site(Some("fr"), false).await.unwrap();
    assert_eq!(spoken_by(fixed, asking("de")).await, "fr");
}
