//! End-to-end security suite: real HTTP requests, in `oneshot`, against an
//! application built by `RuniqueAppBuilder` with its whole middleware stack,
//! its sessions and its admin. Each protection is checked on both sides: the
//! request it refuses and the one it accepts, limit included.

use crate::helpers::db;
use crate::helpers::framework_tables::{
    GROUPES_DDL, GROUPES_DROITS_DDL, HISTORY_DDL, RESET_TOKENS_DDL, SESSIONS_DDL, USERS_DDL,
    USERS_GROUPES_DDL,
};
use crate::helpers::pk::pk;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, Request as HttpRequest, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use runique::app::RuniqueApp;
use runique::auth::permissions::groupes_droits;
use runique::auth::{BuiltinUserEntity, LoginGuard, login, user};
use runique::config::RuniqueConfig;
use runique::context::template::{AppError, Request};
use runique::errors::error::ErrorContext;
use runique::middleware::RateLimiter;
use runique::prelude::{FormAccess, RuniqueForm, ValidationForm, form};
use runique::sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryOrder, Schema, Set,
};
use runique::utils::aliases::AppResult;
use serial_test::serial;
use tower::ServiceExt;

// ── Accounts and data ──────────────────────────────────────────────────────

const PASSWORD: &str = "a correct horse battery";
const ADMIN: u32 = 1;
const LAMBDA: u32 = 2;
const SUPERUSER: u32 = 3;
const DIRIGEANT: u32 = 4;

mod probe {
    use runique::prelude::*;

    model! {
        ProbeNote,
        table: "probe_notes",
        pk: id => i32,
        {
            title: text [required],
            body: text [nullable],
        }
    }
}

mod bare {
    use runique::prelude::*;

    model! {
        BareNote,
        table: "bare_notes",
        pk: id => i32,
        {
            title: text [required],
            body: text [nullable],
        }
    }
}

mod pics {
    use runique::prelude::*;

    model! {
        Pic,
        table: "pics",
        pk: id => i32,
        {
            photo: image [required, upload_to: "pics/"],
        }
    }
}

#[form(schema = pics, fields = All)]
pub struct PicForm;

impl RuniqueForm for PicForm {}

/// Framework tables, the two probe tables, four accounts (one per rank) and
/// the admin's rights: everything on `probe_notes`, nothing on `bare_notes`.
async fn seed(dbc: &DatabaseConnection) {
    for ddl in [
        USERS_DDL,
        GROUPES_DDL,
        GROUPES_DROITS_DDL,
        USERS_GROUPES_DDL,
        HISTORY_DDL,
        RESET_TOKENS_DDL,
        SESSIONS_DDL,
    ] {
        db::exec(dbc, ddl).await;
    }
    let schema = Schema::new(dbc.get_database_backend());
    dbc.execute(&schema.create_table_from_entity(probe::Entity))
        .await
        .expect("probe_notes");
    dbc.execute(&schema.create_table_from_entity(bare::Entity))
        .await
        .expect("bare_notes");

    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let hash = HASH.get_or_init(|| runique::utils::password::hash(PASSWORD).expect("hash"));
    let now = chrono::Utc::now().naive_utc();
    for (n, name, dirigeant, admin, superuser) in [
        (ADMIN, "alice", false, true, false),
        (LAMBDA, "bob", false, false, false),
        (SUPERUSER, "carol", false, false, true),
        (DIRIGEANT, "dora", true, false, false),
    ] {
        user::ActiveModel {
            id: Set(pk(n)),
            username: Set(name.into()),
            email: Set(format!("{name}@example.com")),
            password: Set(hash.clone()),
            is_active: Set(true),
            is_dirigeant: Set(dirigeant),
            is_admin: Set(admin),
            is_superuser: Set(superuser),
            activated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(dbc)
        .await
        .expect("account");
    }
    for (colonne, can_delete) in [("", true), ("title", false)] {
        groupes_droits::ActiveModel {
            groupe_id: Set(None),
            user_id: Set(Some(pk(ADMIN))),
            resource_key: Set("probe_notes".into()),
            colonne: Set(colonne.into()),
            can_create: Set(true),
            can_read: Set(true),
            can_read_own: Set(false),
            can_update: Set(true),
            can_update_own: Set(false),
            can_delete: Set(can_delete),
            can_delete_own: Set(false),
            ..Default::default()
        }
        .insert(dbc)
        .await
        .expect("right");
    }
    probe::ActiveModel {
        id: Set(1),
        title: Set("seed".into()),
        body: Set(Some("kept".into())),
    }
    .insert(dbc)
    .await
    .expect("probe row");
    bare::ActiveModel {
        id: Set(1),
        title: Set("bare".into()),
        body: Set(Some("first".into())),
    }
    .insert(dbc)
    .await
    .expect("bare row");
}

// ── Routes of the test site ────────────────────────────────────────────────

async fn sign_in(Path(n): Path<u32>, req: Request) -> &'static str {
    let account = BuiltinUserEntity::find_by_id(&req.engine.db, pk(n))
        .await
        .expect("seeded account");
    login(&req.session, &account, None, false)
        .await
        .expect("login");
    "ok"
}

async fn whoami(req: Request) -> String {
    req.user
        .as_ref()
        .map_or("anonymous".into(), |u| u.username.clone())
}

/// `title|body` of every row of a table, one per line.
async fn probe_rows(req: Request) -> String {
    probe::Entity::find()
        .order_by_asc(probe::Column::Id)
        .all(&req.engine.db)
        .await
        .unwrap()
        .into_iter()
        .map(|r| format!("{}|{}", r.title, r.body.unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn bare_rows(req: Request) -> String {
    bare::Entity::find()
        .order_by_asc(bare::Column::Id)
        .all(&req.engine.db)
        .await
        .unwrap()
        .into_iter()
        .map(|r| format!("{}|{}", r.title, r.body.unwrap_or_default()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(serde::Deserialize)]
struct Target {
    to: String,
}

/// Redirects wherever the query says, bytes untouched: the open redirect
/// middleware is the only thing between this handler and the browser.
async fn go(Query(target): Query<Target>) -> Response {
    match HeaderValue::from_bytes(target.to.as_bytes()) {
        Ok(location) => (StatusCode::FOUND, [(header::LOCATION, location)]).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn refresh(Query(target): Query<Target>) -> Response {
    match HeaderValue::from_str(&format!("0; url={}", target.to)) {
        Ok(value) => ([(header::REFRESH, value)], "page").into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Validates an image upload through the real form pipeline; nothing is saved.
async fn upload(req: Request) -> String {
    let form = req.form::<PicForm>("pic");
    match ValidationForm::try_form(form, &req).await {
        Ok(_) => "accepted".into(),
        Err(form) => format!("refused: {:?}", form.forms().errors()),
    }
}

/// A database error from a query on a table the error app never creates.
async fn boom_db(req: Request) -> AppResult<Response> {
    let rows = probe::Entity::find()
        .all(&req.engine.db)
        .await
        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    Ok(format!("{} rows", rows.len()).into_response())
}

fn site_routes() -> Router {
    Router::new()
        .route("/sign-in/{n}", get(sign_in))
        .route("/whoami", get(whoami))
        .route("/probe/rows", get(probe_rows))
        .route("/bare/rows", get(bare_rows))
        .route("/go", get(go))
        .route("/refresh", get(refresh))
        .route("/echo", post(|| async { "posted" }))
        .route("/upload", get(upload).post(upload))
        .route("/boom-db", get(boom_db))
}

// ── Building the app ───────────────────────────────────────────────────────

#[derive(Default)]
struct Opts {
    prod: bool,
    debug: bool,
    enforce_https: bool,
    allowed_hosts: Vec<&'static str>,
    rate_limiter: Option<RateLimiter>,
    login_guard: Option<LoginGuard>,
    static_root: Option<std::path::PathBuf>,
    seeded: bool,
}

impl Opts {
    fn seeded() -> Self {
        Self {
            seeded: true,
            ..Default::default()
        }
    }
}

async fn build(opts: Opts) -> Router {
    let dbc = db::fresh_db().await;
    if opts.seeded {
        seed(&dbc).await;
    }
    let mut config = RuniqueConfig::from_env();
    config.dev = !opts.prod;
    config.debug = opts.debug;
    config.security.enforce_https = opts.enforce_https;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    // Uploads are staged under the test run's MEDIA_ROOT, never in the repository.
    config.static_files.media_root = runique::config::static_files::resolve_media_root();
    let statics_on = opts.static_root.is_some();
    if let Some(root) = &opts.static_root {
        config.static_files.staticfiles_dirs = root.join("static").to_string_lossy().into_owned();
        config.static_files.media_root = root.join("media").to_string_lossy().into_owned();
        config.static_files.static_url = "/static".into();
        config.static_files.media_url = "/media".into();
    }
    let hosts = opts.allowed_hosts.clone();
    RuniqueApp::builder(config)
        .with_public_url("https://site.test")
        .with_database(dbc)
        .routes(site_routes())
        .static_files(move |s| s.enabled(statics_on))
        .middleware(move |m| {
            if hosts.is_empty() {
                m
            } else {
                m.with_allowed_hosts(move |h| hosts.into_iter().fold(h, |h, host| h.host(host)))
            }
        })
        .with_admin(move |a| {
            let mut a = a.path("/secret-admin").site(|s| {
                s.register(
                    runique::admin::builder::admin_builder::ModelAdmin::<probe::Entity>::new()
                        .list(|l| l.display([probe::Column::Title]).sortable([probe::Column::Title])),
                )
                .register(runique::admin::builder::admin_builder::ModelAdmin::<bare::Entity>::new())
            });
            if let Some(limiter) = opts.rate_limiter {
                a = a.with_rate_limiter(limiter);
            }
            if let Some(guard) = opts.login_guard {
                a = a.with_login_guard(guard);
            }
            a
        })
        .build()
        .await
        .expect("app")
        .router
}

// ── A client keeping its cookie across oneshot requests ───────────────────

struct Client {
    app: Router,
    cookie: Option<String>,
}

impl Client {
    fn new(app: &Router) -> Self {
        Self {
            app: app.clone(),
            cookie: None,
        }
    }

    async fn send(&mut self, mut req: HttpRequest<Body>) -> Response {
        if let Some(cookie) = &self.cookie {
            req.headers_mut()
                .insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        }
        let resp = self.app.clone().oneshot(req).await.unwrap();
        if let Some(set) = resp.headers().get(header::SET_COOKIE)
            && let Ok(set) = set.to_str()
            && let Some(pair) = set.split(';').next()
        {
            self.cookie = Some(pair.to_string());
        }
        resp
    }

    async fn get(&mut self, uri: &str) -> Response {
        self.get_with(uri, &[]).await
    }

    async fn get_with(&mut self, uri: &str, headers: &[(&str, &str)]) -> Response {
        let mut req = HttpRequest::builder().uri(uri);
        for (name, value) in headers {
            req = req.header(*name, *value);
        }
        self.send(req.body(Body::empty()).unwrap()).await
    }

    async fn post_raw(
        &mut self,
        uri: &str,
        content_type: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
    ) -> Response {
        let mut req = HttpRequest::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, content_type);
        for (name, value) in headers {
            req = req.header(*name, *value);
        }
        self.send(req.body(Body::from(body)).unwrap()).await
    }

    async fn post_form(
        &mut self,
        uri: &str,
        fields: &[(&str, &str)],
        headers: &[(&str, &str)],
    ) -> Response {
        let body = serde_urlencoded::to_string(fields).unwrap();
        self.post_raw(
            uri,
            "application/x-www-form-urlencoded",
            body.into_bytes(),
            headers,
        )
        .await
    }

    async fn text(&mut self, uri: &str) -> String {
        body(self.get(uri).await).await
    }

    async fn sign_in(&mut self, n: u32) {
        assert_eq!(self.text(&format!("/sign-in/{n}")).await, "ok");
    }

    /// The masked CSRF token the app hands this session.
    async fn token(&mut self) -> String {
        let resp = self.get("/whoami").await;
        header_of(&resp, "x-csrf-token")
    }
}

async fn body(resp: Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn header_of(resp: &Response, name: &str) -> String {
    resp.headers()
        .get(name)
        .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
        .unwrap_or_default()
}

fn location(resp: &Response) -> String {
    header_of(resp, "location")
}

/// The admin's own session: signed in through the site, with its token.
async fn admin_client(app: &Router, n: u32) -> (Client, String) {
    let mut client = Client::new(app);
    client.sign_in(n).await;
    let token = client.token().await;
    (client, token)
}

// ══════════════════════════════════════════════════════════════════════════
// 1. Access and rights
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn only_ranks_above_lambda_enter_the_admin() {
    let app = build(Opts::seeded()).await;
    for (n, why) in [
        (ADMIN, "admin"),
        (DIRIGEANT, "dirigeant"),
        (SUPERUSER, "superuser"),
    ] {
        let mut client = Client::new(&app);
        client.sign_in(n).await;
        let resp = client.get("/secret-admin/").await;
        assert_eq!(resp.status(), StatusCode::OK, "{why} reaches the dashboard");
    }

    // A signed-in lambda is sent home, not to a login page that would change
    // nothing; an anonymous visitor is sent to the login page.
    let mut lambda = Client::new(&app);
    lambda.sign_in(LAMBDA).await;
    for uri in [
        "/secret-admin/",
        "/secret-admin/probe_notes/list",
        "/secret-admin/probe_notes/1/detail",
        "/secret-admin/probe_notes/1/edit",
        "/secret-admin/probe_notes/1/delete",
    ] {
        let resp = lambda.get(uri).await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER, "{uri}");
        assert_eq!(location(&resp), "/", "{uri}");
    }
    assert_eq!(
        lambda.text("/whoami").await,
        "bob",
        "the refusal keeps the site session"
    );

    let mut anonymous = Client::new(&app);
    let resp = anonymous.get("/secret-admin/probe_notes/1/detail").await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), "/secret-admin/login");
}

/// IDOR: the admin holds rights on `probe_notes` only. Naming a row of
/// `bare_notes` in the URL gives neither the page nor the write.
#[tokio::test]
#[serial]
async fn an_admin_without_the_right_on_a_table_can_neither_read_nor_write_its_rows() {
    let app = build(Opts::seeded()).await;
    let (mut admin, token) = admin_client(&app, ADMIN).await;

    for page in ["detail", "edit", "delete"] {
        let resp = admin
            .get(&format!("/secret-admin/bare_notes/1/{page}"))
            .await;
        assert_eq!(resp.status(), StatusCode::SEE_OTHER, "{page} refused");
        assert_eq!(location(&resp), "/secret-admin/", "{page} refused");

        let resp = admin
            .get(&format!("/secret-admin/probe_notes/1/{page}"))
            .await;
        assert_eq!(resp.status(), StatusCode::OK, "{page} allowed");
    }
    let list = admin.get("/secret-admin/bare_notes/list").await;
    assert_eq!(location(&list), "/secret-admin/", "list refused");
    assert_eq!(
        admin.get("/secret-admin/probe_notes/list").await.status(),
        StatusCode::OK
    );

    let forged = admin
        .post_form(
            "/secret-admin/bare_notes/1/edit",
            &[
                ("csrf_token", &token),
                ("runique_form", "admin_update"),
                ("title", "hacked"),
                ("body", "hacked"),
            ],
            &[],
        )
        .await;
    assert_eq!(location(&forged), "/secret-admin/");
    assert_eq!(admin.text("/bare/rows").await, "bare|first");

    let forged = admin
        .post_form(
            "/secret-admin/bare_notes/1/delete",
            &[("csrf_token", &token), ("runique_form", "admin_delete")],
            &[],
        )
        .await;
    assert_eq!(location(&forged), "/secret-admin/");
    assert_eq!(admin.text("/bare/rows").await, "bare|first");

    let saved = admin
        .post_form(
            "/secret-admin/probe_notes/1/edit",
            &[
                ("csrf_token", &token),
                ("runique_form", "admin_update"),
                ("title", "changed"),
            ],
            &[],
        )
        .await;
    assert_eq!(saved.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&saved), "/secret-admin/probe_notes/1/detail");
    assert_eq!(admin.text("/probe/rows").await, "changed|kept");

    let deleted = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[("csrf_token", &token), ("runique_form", "admin_delete")],
            &[],
        )
        .await;
    assert_eq!(location(&deleted), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "");
}

/// The superuser, whose rights need no grid, reaches the table the admin can't.
#[tokio::test]
#[serial]
async fn the_superuser_reads_and_writes_the_table_the_admin_has_no_right_on() {
    let app = build(Opts::seeded()).await;
    let (mut su, token) = admin_client(&app, SUPERUSER).await;
    assert_eq!(
        su.get("/secret-admin/bare_notes/1/detail").await.status(),
        StatusCode::OK
    );
    let saved = su
        .post_form(
            "/secret-admin/bare_notes/1/edit",
            &[
                ("csrf_token", &token),
                ("runique_form", "admin_update"),
                ("title", "by carol"),
                ("body", "first"),
            ],
            &[],
        )
        .await;
    assert_eq!(location(&saved), "/secret-admin/bare_notes/1/detail");
    assert_eq!(su.text("/bare/rows").await, "by carol|first");
}

// ══════════════════════════════════════════════════════════════════════════
// 2. Session
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn the_session_id_changes_at_login_so_a_planted_one_opens_nothing() {
    let app = build(Opts::seeded()).await;
    let mut client = Client::new(&app);
    client.get("/whoami").await;
    let planted = client.cookie.clone().expect("anonymous session cookie");

    client.sign_in(ADMIN).await;
    let signed = client.cookie.clone().expect("session cookie after login");
    assert_ne!(planted, signed, "the id is rotated at login");
    assert_eq!(client.text("/whoami").await, "alice");

    let mut attacker = Client::new(&app);
    attacker.cookie = Some(planted);
    assert_eq!(
        attacker.text("/whoami").await,
        "anonymous",
        "the planted id is not the signed-in session"
    );
    let resp = attacker.get("/secret-admin/").await;
    assert_eq!(location(&resp), "/secret-admin/login");
}

#[tokio::test]
#[serial]
async fn a_closed_session_opens_nothing_any_more() {
    let app = build(Opts::seeded()).await;
    let (mut admin, token) = admin_client(&app, ADMIN).await;
    let cookie = admin.cookie.clone().expect("cookie");
    assert_eq!(
        admin.get("/secret-admin/").await.status(),
        StatusCode::OK,
        "open before logout"
    );

    let out = admin
        .post_form("/secret-admin/logout", &[("csrf_token", &token)], &[])
        .await;
    assert_eq!(out.status(), StatusCode::SEE_OTHER);

    let mut holder = Client::new(&app);
    holder.cookie = Some(cookie);
    assert_eq!(holder.text("/whoami").await, "anonymous");
    let resp = holder.get("/secret-admin/").await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), "/secret-admin/login");
    let resp = holder.get("/secret-admin/probe_notes/1/detail").await;
    assert_eq!(location(&resp), "/secret-admin/login");
}

#[tokio::test]
#[serial]
async fn the_session_cookie_is_http_only_strict_and_secure_in_production() {
    let app = build(Opts::seeded()).await;
    let resp = Client::new(&app).get("/whoami").await;
    let cookie = header_of(&resp, "set-cookie");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
    assert!(!cookie.contains("Secure"), "development: {cookie}");

    let prod = build(Opts {
        prod: true,
        enforce_https: true,
        ..Opts::seeded()
    })
    .await;
    let resp = Client::new(&prod).get("/whoami").await;
    let cookie = header_of(&resp, "set-cookie");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
    assert!(cookie.contains("Secure"), "production: {cookie}");
}

// ══════════════════════════════════════════════════════════════════════════
// 3. CSRF
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn a_post_without_a_token_or_with_another_sessions_token_writes_nothing() {
    let app = build(Opts::seeded()).await;
    let (mut admin, _) = admin_client(&app, ADMIN).await;
    let (mut other, other_token) = admin_client(&app, SUPERUSER).await;

    let no_token = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[("runique_form", "admin_delete")],
            &[],
        )
        .await;
    assert_ne!(location(&no_token), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "seed|kept");

    let stolen = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[
                ("csrf_token", &other_token),
                ("runique_form", "admin_delete"),
            ],
            &[],
        )
        .await;
    assert_ne!(location(&stolen), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "seed|kept");
    assert_eq!(
        other.text("/whoami").await,
        "carol",
        "the other session is untouched"
    );

    let garbage = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[("csrf_token", "not-a-token"), ("runique_form", "admin_delete")],
            &[],
        )
        .await;
    assert_ne!(location(&garbage), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "seed|kept");

    let own_token = admin.token().await;
    let ok = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[("csrf_token", &own_token), ("runique_form", "admin_delete")],
            &[],
        )
        .await;
    assert_eq!(location(&ok), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "");
}

#[tokio::test]
#[serial]
async fn a_form_announced_cross_site_or_same_site_is_refused_before_any_handler() {
    let app = build(Opts::seeded()).await;
    let (mut admin, token) = admin_client(&app, ADMIN).await;

    for site in ["cross-site", "same-site", "Cross-Site"] {
        let resp = admin
            .post_form(
                "/secret-admin/probe_notes/1/delete",
                &[("csrf_token", &token), ("runique_form", "admin_delete")],
                &[("sec-fetch-site", site)],
            )
            .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{site}");
        assert_eq!(admin.text("/probe/rows").await, "seed|kept", "{site}");
    }

    let resp = admin
        .post_form(
            "/secret-admin/probe_notes/1/delete",
            &[("csrf_token", &token), ("runique_form", "admin_delete")],
            &[("sec-fetch-site", "same-origin")],
        )
        .await;
    assert_eq!(location(&resp), "/secret-admin/probe_notes/list");
    assert_eq!(admin.text("/probe/rows").await, "");
}

/// The middleware alone, on a bare route: a non-form body needs the header,
/// a wrong header is refused, the session's own masked token is accepted.
#[tokio::test]
#[serial]
async fn a_non_form_post_needs_the_sessions_token_in_its_header() {
    let app = build(Opts::seeded()).await;
    let mut client = Client::new(&app);
    let token = client.token().await;
    let mut other = Client::new(&app);
    let other_token = other.token().await;

    let json = b"{}".to_vec();
    let resp = client
        .post_raw("/echo", "application/json", json.clone(), &[])
        .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "no header");

    let resp = client
        .post_raw(
            "/echo",
            "application/json",
            json.clone(),
            &[("x-csrf-token", "garbage")],
        )
        .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "garbage header");

    let resp = client
        .post_raw(
            "/echo",
            "application/json",
            json.clone(),
            &[("x-csrf-token", &other_token)],
        )
        .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN, "another session's token");

    let resp = client
        .post_raw(
            "/echo",
            "application/json",
            json,
            &[("x-csrf-token", &token)],
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body(resp).await, "posted");
}

// ══════════════════════════════════════════════════════════════════════════
// 4. Open redirect
// ══════════════════════════════════════════════════════════════════════════

fn encoded(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

#[tokio::test]
#[serial]
async fn a_redirect_leaving_the_site_is_replaced_by_a_400() {
    let app = build(Opts {
        allowed_hosts: vec!["allowed.com"],
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);

    for payload in [
        "//evil.com",
        "/\\evil.com",
        "/\t/evil.com",
        "https:evil.com",
        "https://allowed.com:x@evil.com",
        "https://allowed.com@evil.com",
        "https://evil.com/",
        "http://evil.com",
        "//evil\u{FF0E}com/",
        "javascript:alert(1)",
        "http://127.evil.com/",
    ] {
        let resp = client.get(&format!("/go?to={}", encoded(payload))).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{payload:?}");
        assert!(resp.headers().get("location").is_none(), "{payload:?}");
    }

    for payload in [
        "/dashboard",
        "/caf\u{E9}/menu",
        "https://allowed.com/path",
        "http://localhost:3000/x",
    ] {
        let resp = client.get(&format!("/go?to={}", encoded(payload))).await;
        assert_eq!(resp.status(), StatusCode::FOUND, "{payload:?}");
        assert_eq!(location(&resp), payload, "{payload:?}");
    }
}

#[tokio::test]
#[serial]
async fn a_refresh_header_leaving_the_site_is_refused_too() {
    let app = build(Opts {
        allowed_hosts: vec!["allowed.com"],
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    for payload in ["https://evil.com", "//evil.com", "'//evil.com'"] {
        let resp = client
            .get(&format!("/refresh?to={}", encoded(payload)))
            .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{payload:?}");
    }
    for payload in ["/home", "https://allowed.com/home"] {
        let resp = client
            .get(&format!("/refresh?to={}", encoded(payload)))
            .await;
        assert_eq!(resp.status(), StatusCode::OK, "{payload:?}");
        assert_eq!(header_of(&resp, "refresh"), format!("0; url={payload}"));
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 5. Headers and hosts
// ══════════════════════════════════════════════════════════════════════════

fn nonce_of(csp: &str) -> String {
    let start = csp.find("'nonce-").expect("a nonce in the CSP") + "'nonce-".len();
    let end = csp[start..].find('\'').unwrap() + start;
    csp[start..end].to_string()
}

#[tokio::test]
#[serial]
async fn every_response_carries_a_fresh_csp_nonce_and_denies_framing() {
    let app = build(Opts::seeded()).await;
    let mut client = Client::new(&app);
    let first = client.get("/whoami").await;
    let second = client.get("/whoami").await;
    let (csp1, csp2) = (
        header_of(&first, "content-security-policy"),
        header_of(&second, "content-security-policy"),
    );
    assert!(csp1.contains("script-src 'self' 'nonce-"), "{csp1}");
    assert!(csp1.contains("frame-ancestors 'none'"), "{csp1}");
    assert!(!csp1.contains("'unsafe-inline'"), "{csp1}");
    let (n1, n2) = (nonce_of(&csp1), nonce_of(&csp2));
    assert!(n1.len() >= 16, "{n1}");
    assert_ne!(n1, n2, "one nonce per response");
    assert_eq!(header_of(&first, "x-frame-options"), "DENY");
    assert_eq!(header_of(&first, "x-content-type-options"), "nosniff");

    let (mut admin, _) = admin_client(&app, ADMIN).await;
    let page = admin.get("/secret-admin/").await;
    assert_eq!(header_of(&page, "x-frame-options"), "DENY");
    assert_eq!(header_of(&page, "cache-control"), "no-store");
}

#[tokio::test]
#[serial]
async fn hsts_is_sent_only_when_https_is_enforced() {
    let plain = build(Opts {
        prod: true,
        ..Opts::seeded()
    })
    .await;
    let resp = Client::new(&plain).get("/whoami").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers().get("strict-transport-security").is_none(),
        "no HSTS over plain HTTP"
    );

    let https = build(Opts {
        prod: true,
        enforce_https: true,
        ..Opts::seeded()
    })
    .await;
    let resp = Client::new(&https).get("/whoami").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let hsts = header_of(&resp, "strict-transport-security");
    assert!(hsts.starts_with("max-age="), "{hsts}");

    let resp = Client::new(&https)
        .get_with("/whoami", &[("x-forwarded-proto", "http"), ("host", "site.test")])
        .await;
    assert!(resp.status().is_redirection(), "forwarded over http");
    assert!(location(&resp).starts_with("https://site.test"), "{}", location(&resp));
}

#[tokio::test]
#[serial]
async fn a_host_outside_the_allowlist_is_refused() {
    let app = build(Opts {
        prod: true,
        allowed_hosts: vec!["allowed.test"],
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    for host in ["evil.test", "allowed.test.evil.test", "xallowed.test"] {
        let resp = client.get_with("/whoami", &[("host", host)]).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "{host}");
    }
    let resp = client.get("/whoami").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST, "no Host header");

    for host in ["allowed.test", "allowed.test:8443"] {
        let resp = client.get_with("/whoami", &[("host", host)]).await;
        assert_eq!(resp.status(), StatusCode::OK, "{host}");
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 6. Rate limiting and login guard
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn the_rate_limiter_answers_429_past_its_ceiling() {
    let app = build(Opts {
        rate_limiter: Some(RateLimiter::new().max_requests(3).retry_after(60)),
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    let fields = [("username", "alice"), ("password", "wrong")];
    for attempt in 1..=3 {
        let resp = client.post_form("/secret-admin/login", &fields, &[]).await;
        assert_eq!(resp.status(), StatusCode::OK, "attempt {attempt} is served");
    }
    let resp = client.post_form("/secret-admin/login", &fields, &[]).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry = header_of(&resp, "retry-after");
    assert!(retry.parse::<u64>().is_ok_and(|s| s > 0 && s <= 60), "{retry}");

    assert_eq!(
        client.get("/secret-admin/login").await.status(),
        StatusCode::OK,
        "the limit is on the POST only"
    );
}

async fn login_attempt(client: &mut Client, password: &str) -> Response {
    let page = client.get("/secret-admin/login").await;
    let token = header_of(&page, "x-csrf-token");
    client
        .post_form(
            "/secret-admin/login",
            &[
                ("csrf_token", &token),
                ("username", "alice"),
                ("password", password),
            ],
            &[],
        )
        .await
}

#[tokio::test]
#[serial]
async fn the_login_guard_locks_the_account_after_n_failures() {
    let guard = || LoginGuard::new().max_attempts(3).lockout_secs(300);

    let app = build(Opts {
        login_guard: Some(guard()),
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    for _ in 0..2 {
        let resp = login_attempt(&mut client, "wrong").await;
        assert_eq!(resp.status(), StatusCode::OK, "login page shown again");
    }
    let resp = login_attempt(&mut client, PASSWORD).await;
    assert_eq!(resp.status(), StatusCode::SEE_OTHER, "two failures: still open");
    assert_eq!(location(&resp), "/secret-admin/");
    assert_eq!(client.text("/whoami").await, "alice");

    let app = build(Opts {
        login_guard: Some(guard()),
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    for _ in 0..3 {
        login_attempt(&mut client, "wrong").await;
    }
    let resp = login_attempt(&mut client, PASSWORD).await;
    assert_eq!(resp.status(), StatusCode::OK, "locked: no redirect");
    let page = body(resp).await;
    assert!(
        page.contains("Too many attempts") || page.contains("Trop de tentatives"),
        "{page}"
    );
    assert_eq!(client.text("/whoami").await, "anonymous");

    let mut attacker = Client::new(&app);
    let resp = login_attempt(&mut attacker, PASSWORD).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "the lock is on the account, not the session"
    );
}

// ══════════════════════════════════════════════════════════════════════════
// 7. Inputs
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn the_admin_search_echoes_its_value_escaped() {
    let app = build(Opts::seeded()).await;
    let (mut su, _) = admin_client(&app, SUPERUSER).await;
    let payload = "<script>alert(1)</script>\"><img src=x onerror=alert(2)>";
    let resp = su
        .get(&format!(
            "/secret-admin/probe_notes/list?search={}",
            encoded(payload)
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let page = body(resp).await;
    assert!(!page.contains("<script>alert(1)"), "{page}");
    assert!(!page.contains("<img src=x"), "{page}");
    assert!(page.contains("&lt;script&gt;alert(1)"), "escaped: {page}");

    let plain = su.get("/secret-admin/probe_notes/list?search=seed").await;
    assert!(body(plain).await.contains("value=\"seed\""));
}

fn static_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("rq_sec_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(root.join("static")).unwrap();
    std::fs::create_dir_all(root.join("media")).unwrap();
    std::fs::write(root.join("static/app.css"), "body{}").unwrap();
    std::fs::write(root.join("media/photo.txt"), "x").unwrap();
    std::fs::write(root.join("secret.txt"), "TOP-SECRET").unwrap();
    root
}

#[tokio::test]
#[serial]
async fn dot_dot_segments_never_leave_the_static_or_media_folders() {
    let root = static_root();
    let app = build(Opts {
        static_root: Some(root.clone()),
        ..Opts::seeded()
    })
    .await;
    let mut client = Client::new(&app);
    for uri in [
        "/static/../secret.txt",
        "/static/%2e%2e/secret.txt",
        "/static/..%2fsecret.txt",
        "/static/%2e%2e%2fsecret.txt",
        "/static/.%2e/secret.txt",
        "/media/../secret.txt",
        "/media/..%2fsecret.txt",
        "/media/%2e%2e/secret.txt",
        "/runique/static/../../tests/app/mod.rs",
    ] {
        let resp = client.get(uri).await;
        let status = resp.status();
        let text = body(resp).await;
        assert_ne!(status, StatusCode::OK, "{uri}: {text}");
        assert!(!text.contains("TOP-SECRET"), "{uri}");
        assert!(!text.contains("pub mod"), "{uri}");
    }
    assert_eq!(client.get("/static/app.css").await.status(), StatusCode::OK);
    assert_eq!(client.get("/media/photo.txt").await.status(), StatusCode::OK);
    let _ = std::fs::remove_dir_all(root);
}

async fn upload_named(client: &mut Client, filename: &str, content: &[u8]) -> String {
    let token = client.token().await;
    let b = "BNDRY";
    let mut bytes = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"csrf_token\"\r\n\r\n{token}\r\n\
         --{b}\r\nContent-Disposition: form-data; name=\"runique_form\"\r\n\r\npic\r\n\
         --{b}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"{filename}\"\r\n\
         Content-Type: image/png\r\n\r\n"
    )
    .into_bytes();
    bytes.extend_from_slice(content);
    bytes.extend_from_slice(format!("\r\n--{b}--\r\n").as_bytes());
    let resp = client
        .post_raw(
            "/upload",
            &format!("multipart/form-data; boundary={b}"),
            bytes,
            &[("x-csrf-token", &token)],
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{filename}");
    body(resp).await
}

#[tokio::test]
#[serial]
async fn an_html_or_svg_file_disguised_as_an_image_is_refused() {
    let app = build(Opts::seeded()).await;
    let mut client = Client::new(&app);
    let png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    assert_eq!(upload_named(&mut client, "photo.png", &png).await, "accepted");

    for (name, content) in [
        ("page.png", &b"<html><script>alert(1)</script></html>"[..]),
        ("logo.png", &b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>"[..]),
        ("logo.svg", &b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"[..]),
        ("page.html", &b"<html></html>"[..]),
        ("tiny.png", &b"\x89PN"[..]),
        ("real-png.svg", &png[..]),
    ] {
        let verdict = upload_named(&mut client, name, content).await;
        assert!(verdict.starts_with("refused"), "{name}: {verdict}");
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 8. Error pages in production
// ══════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn production_error_pages_show_neither_sql_nor_path_nor_trace() {
    let prod = build(Opts {
        prod: true,
        debug: false,
        ..Default::default()
    })
    .await;
    let mut client = Client::new(&prod);
    let resp = client.get("/boom-db").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let page = body(resp).await;
    for leak in [
        "probe_notes",
        "no such table",
        "SELECT",
        "DbErr",
        "runique/src",
        "/home/",
        "stack trace",
    ] {
        assert!(!page.contains(leak), "{leak} leaked: {page}");
    }

    let resp = client.get("/nowhere/secret-probe-path").await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let page = body(resp).await;
    assert!(!page.contains("secret-probe-path"), "{page}");

    let debug = build(Opts {
        debug: true,
        ..Default::default()
    })
    .await;
    let resp = Client::new(&debug).get("/boom-db").await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let page = body(resp).await;
    assert!(
        page.contains("no such table") || page.contains("probe_notes"),
        "the debug page does tell: {page}"
    );
}
