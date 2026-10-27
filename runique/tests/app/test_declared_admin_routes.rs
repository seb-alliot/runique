//! Tests — the routes declared `admin` in `urlpatterns!`: their boundary, which
//! lives at the site's own paths (a refusal must neither show the admin's
//! prefix to a stranger nor close a site session), the `Detail` override, and
//! the forms the macro accepts.

use crate::helpers::db;
use crate::helpers::framework_tables::{
    GROUPES_DDL, GROUPES_DROITS_DDL, HISTORY_DDL, USERS_DDL, USERS_GROUPES_DDL,
};
use crate::helpers::pk::{pk, pk_sql_literal};
use axum::extract::Path;
use axum::routing::get;
use runique::app::RuniqueApp;
use runique::auth::{BuiltinUserEntity, login};
use runique::config::RuniqueConfig;
use runique::context::template::Request;
use runique::flash::Message;
use runique::macros::Routes;
use runique::prelude::{FormAccess, RuniqueForm, ValidationForm, form};
use runique::sea_orm::{EntityTrait, QueryOrder};
use serial_test::serial;

async fn sign_in(Path(n): Path<u32>, req: Request) -> &'static str {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(n))
        .await
        .expect("seeded account");
    login(&req.session, &user, None, false)
        .await
        .expect("login");
    "ok"
}

async fn whoami(req: Request) -> String {
    req.user.map_or("anonymous".into(), |u| u.username)
}

async fn flashes(notices: Message) -> String {
    notices
        .get_all()
        .await
        .into_iter()
        .map(|m| m.content)
        .collect::<Vec<_>>()
        .join("|")
}

const ADMIN: u32 = 1;
const MEMBER: u32 = 2;
const SUPERUSER: u32 = 3;
const STAFF_MEMBER: u32 = 4;
const ORPHAN_STAFF_MEMBER: u32 = 5;
const IDLE_STAFF_MEMBER: u32 = 6;

/// Accounts: an admin, a member without any group, a superuser, and three
/// members of a staff group — one whose responsable is the admin (the group
/// counts), one whose responsable is a plain member (it doesn't: rule A), one
/// whose group is disabled.
async fn seed(dbc: &runique::sea_orm::DatabaseConnection) {
    for ddl in [
        USERS_DDL,
        GROUPES_DDL,
        GROUPES_DROITS_DDL,
        USERS_GROUPES_DDL,
        HISTORY_DDL,
        "CREATE TABLE probe_notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL)",
    ] {
        db::exec(dbc, ddl).await;
    }
    let account = |n: u32, name: &str, flags: &str| {
        format!(
            "({}, '{name}', '{name}@example.com', 'x', 1, {flags}, '2026-01-01 00:00:00')",
            pk_sql_literal(n)
        )
    };
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_users \
             (id, username, email, password, is_active, is_admin, is_superuser, activated_at) VALUES {}",
            [
                account(ADMIN, "alice", "1, 0"),
                account(MEMBER, "bob", "0, 0"),
                account(SUPERUSER, "carol", "0, 1"),
                account(STAFF_MEMBER, "dave", "0, 0"),
                account(ORPHAN_STAFF_MEMBER, "eve", "0, 0"),
                account(IDLE_STAFF_MEMBER, "frank", "0, 0"),
            ]
            .join(", ")
        ),
    )
    .await;
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_groupes (id, nom, is_staff, is_active, responsable_id) VALUES \
             (1, 'relecteurs', 1, 1, {}), (2, 'orphelins', 1, 1, {}), (3, 'en pause', 1, 0, {})",
            pk_sql_literal(ADMIN),
            pk_sql_literal(MEMBER),
            pk_sql_literal(ADMIN)
        ),
    )
    .await;
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({}, 1), ({}, 2), ({}, 3)",
            pk_sql_literal(STAFF_MEMBER),
            pk_sql_literal(ORPHAN_STAFF_MEMBER),
            pk_sql_literal(IDLE_STAFF_MEMBER)
        ),
    )
    .await;
    db::exec(dbc, "INSERT INTO probe_notes (id, title) VALUES (1, 'seed')").await;
    // The admin's grid on `probe_notes`: create, update, delete, `title`
    // ticked. The staff group (its responsable is the admin): update only.
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_groupes_droits \
             (groupe_id, user_id, resource_key, colonne, can_create, can_read, can_update, can_delete) VALUES \
             (NULL, {admin}, 'probe_notes', '', 1, 1, 1, 1), \
             (NULL, {admin}, 'probe_notes', 'title', 1, 1, 1, 0), \
             (1, NULL, 'probe_notes', '', 0, 1, 1, 0), \
             (1, NULL, 'probe_notes', 'title', 0, 1, 1, 0)",
            admin = pk_sql_literal(ADMIN)
        ),
    )
    .await;
}

fn routes() -> Routes {
    runique::urlpatterns! {
        "/sign-in/{n}" => get(sign_in), name = "declared_test_sign_in",
        "/whoami" => get(whoami), name = "declared_test_whoami",
        "/flashes" => get(flashes), name = "declared_test_flashes",
        admin "/custom/{id}" => get(|| async { "private" }),
            name = "declared_test_custom",
            for_model = runique::auth::user::Entity,
        admin "/accounts/{id}" => get(|| async { "custom detail" }),
            name = "declared_test_account_detail",
            for_model = runique::auth::user::Entity,
            overrides = runique::admin::AdminView::Detail,
    }
}

/// The app served on a free port, the admin at `{mount}/secret-admin`.
async fn spawn_with(mount: &'static str, routes: Routes) -> String {
    let dbc = db::fresh_db().await;
    seed(&dbc).await;

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(dbc)
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .with_admin(|a| {
            let a = a.path("/secret-admin").site(|s| {
                s.register(runique::admin::builder::admin_builder::ModelAdmin::<
                    probe::Entity,
                >::new())
            });
            if mount.is_empty() { a } else { a.prefix(mount) }
        })
        .build()
        .await
        .expect("app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app.router).await.unwrap() });
    format!("http://{addr}")
}

async fn spawn() -> String {
    spawn_with("", routes()).await
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

fn location(resp: &reqwest::Response) -> Option<&str> {
    resp.headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
}

async fn fetch(client: &reqwest::Client, url: String, referer: Option<&str>) -> reqwest::Response {
    let mut req = client.get(url);
    if let Some(referer) = referer {
        req = req.header(reqwest::header::REFERER, referer);
    }
    req.send().await.unwrap()
}

async fn text(client: &reqwest::Client, url: String) -> String {
    fetch(client, url, None).await.text().await.unwrap()
}

/// A client signed in as account `n`.
async fn signed_in(base: &str, n: u32) -> reqwest::Client {
    let client = client();
    assert_eq!(text(&client, format!("{base}/sign-in/{n}")).await, "ok");
    client
}

// ── Anonymous visitors ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn an_anonymous_visitor_goes_home_without_seeing_the_admin_prefix() {
    let base = spawn().await;
    let resp = fetch(&client(), format!("{base}/custom/42"), None).await;
    assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), Some("/"));
}

#[tokio::test]
#[serial]
async fn coming_back_from_an_admin_page_leads_to_the_admin_login() {
    let base = spawn().await;
    let referer = format!("{base}/secret-admin/users/list");
    let resp = fetch(&client(), format!("{base}/custom/42"), Some(&referer)).await;
    assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(location(&resp), Some("/secret-admin/login"));
}

#[tokio::test]
#[serial]
async fn a_referer_from_a_public_page_goes_home() {
    let base = spawn().await;
    let referer = format!("{base}/whoami");
    let resp = fetch(&client(), format!("{base}/custom/42"), Some(&referer)).await;
    assert_eq!(location(&resp), Some("/"));
}

/// With a mount prefix, the admin's public prefix is the composed one.
#[tokio::test]
#[serial]
async fn the_mount_prefix_is_part_of_the_admin_prefix() {
    let base = spawn_with("mnt", routes()).await;

    let referer = format!("{base}/mnt/secret-admin/");
    let resp = fetch(&client(), format!("{base}/custom/42"), Some(&referer)).await;
    assert_eq!(location(&resp), Some("/mnt/secret-admin/login"));

    // Without the mount, the path isn't the admin's.
    let referer = format!("{base}/secret-admin/");
    let resp = fetch(&client(), format!("{base}/custom/42"), Some(&referer)).await;
    assert_eq!(location(&resp), Some("/"));
}

// ── Signed-in accounts ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn an_admin_reaches_the_declared_route() {
    let base = spawn().await;
    let client = signed_in(&base, ADMIN).await;
    let resp = fetch(&client, format!("{base}/custom/42"), None).await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "private");
}

/// The use case of a group page: a member of a staff group whose responsable
/// still holds an admin rank.
#[tokio::test]
#[serial]
async fn a_member_of_a_staff_group_reaches_the_declared_route() {
    let base = spawn().await;
    let client = signed_in(&base, STAFF_MEMBER).await;
    let resp = fetch(&client, format!("{base}/custom/42"), None).await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn accounts_without_admin_access_go_home_with_a_notice_and_stay_signed_in() {
    let base = spawn().await;
    for (n, name, why) in [
        (MEMBER, "bob", "no group"),
        (
            ORPHAN_STAFF_MEMBER,
            "eve",
            "group whose responsable is no admin",
        ),
        (IDLE_STAFF_MEMBER, "frank", "disabled group"),
    ] {
        let client = signed_in(&base, n).await;
        let resp = fetch(&client, format!("{base}/custom/42"), None).await;
        assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER, "{why}");
        assert_eq!(location(&resp), Some("/"), "{why}");

        let notices = text(&client, format!("{base}/flashes")).await;
        assert_eq!(
            notices,
            runique::utils::trad::t("admin.access.action_not_allowed"),
            "{why}"
        );
        assert_eq!(
            text(&client, format!("{base}/whoami")).await,
            name,
            "{why}: the refusal must not close the site session"
        );
    }
}

/// Like every admin page: never cached, never indexed.
#[tokio::test]
#[serial]
async fn a_declared_route_is_neither_cached_nor_indexed() {
    let base = spawn().await;
    let client = signed_in(&base, ADMIN).await;
    let resp = fetch(&client, format!("{base}/custom/42"), None).await;
    let header = |name| {
        resp.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    assert_eq!(header("cache-control").as_deref(), Some("no-store"));
    assert_eq!(header("x-robots-tag").as_deref(), Some("noindex, nofollow"));
}

// ── Detail override ────────────────────────────────────────────────────────

/// The list links each row to the overriding route, and the generated detail
/// page is closed so it can't be used instead.
#[tokio::test]
#[serial]
async fn a_detail_override_replaces_the_generated_page() {
    let base = spawn().await;
    let client = signed_in(&base, SUPERUSER).await;

    let list = fetch(&client, format!("{base}/secret-admin/users/list"), None).await;
    assert_eq!(list.status(), reqwest::StatusCode::OK);
    let body = list.text().await.unwrap();
    for n in [ADMIN, SUPERUSER] {
        let link = format!("/accounts/{}", pk(n));
        assert!(body.contains(&link), "row link to the override: {link}");
    }

    let generated = fetch(
        &client,
        format!("{base}/secret-admin/users/{}/detail", pk(ADMIN)),
        None,
    )
    .await;
    assert_eq!(generated.status(), reqwest::StatusCode::NOT_FOUND);

    assert_eq!(
        text(&client, format!("{base}/accounts/1")).await,
        "custom detail"
    );
}

// ── Build checks ───────────────────────────────────────────────────────────

/// An admin route can't take a name a public route already holds.
#[tokio::test]
#[serial]
async fn an_admin_route_name_already_taken_refuses_the_build() {
    let routes = runique::urlpatterns! {
        "/public" => get(|| async { "" }), name = "declared_test_taken",
        admin "/private" => get(|| async { "" }),
            name = "declared_test_taken",
            for_model = runique::auth::user::Entity,
    };
    let dbc = db::fresh_db().await;
    seed(&dbc).await;
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let error = RuniqueApp::builder(config)
        .with_database(dbc)
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .with_admin(|a| a.path("/secret-admin"))
        .build()
        .await
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        error.contains("conflicts with an already registered URL"),
        "{error}"
    );
}

// ── Forms of `urlpatterns!` ────────────────────────────────────────────────

/// Every form the macro accepts, each last in its set without a trailing comma:
/// a plain route, a named one, an admin route with and without `overrides`.
#[tokio::test]
#[serial]
async fn every_form_of_the_macro_is_served() {
    let routes = runique::urlpatterns! {
        "/sign-in/{n}" => get(sign_in), name = "declared_test_forms_sign_in",
        "/plain" => get(|| async { "plain" })
    }
    .merge(runique::urlpatterns! {
        "/named" => get(|| async { "named" }), name = "declared_test_forms_named"
    })
    .merge(runique::urlpatterns! {
        admin "/standalone" => get(|| async { "standalone" }),
            name = "declared_test_forms_standalone",
            for_model = runique::auth::user::Entity
    })
    .merge(runique::urlpatterns! {
        admin "/detail/{id}" => get(|| async { "detail" }),
            name = "declared_test_forms_detail",
            for_model = runique::auth::user::Entity,
            overrides = runique::admin::AdminView::Detail
    });
    let base = spawn_with("", routes).await;

    let anonymous = client();
    assert_eq!(text(&anonymous, format!("{base}/plain")).await, "plain");
    assert_eq!(text(&anonymous, format!("{base}/named")).await, "named");
    // Behind the admin boundary: an anonymous visitor is sent home.
    for path in ["/standalone", "/detail/1"] {
        let resp = fetch(&anonymous, format!("{base}{path}"), None).await;
        assert_eq!(location(&resp), Some("/"), "{path}");
    }

    let admin = signed_in(&base, ADMIN).await;
    assert_eq!(
        text(&admin, format!("{base}/standalone")).await,
        "standalone"
    );
    assert_eq!(text(&admin, format!("{base}/detail/1")).await, "detail");
}

// ── Writing through a form inside the admin's constraint ──────────────────

mod probe {
    use runique::prelude::*;

    model! {
        ProbeNote,
        table: "probe_notes",
        pk: id => i32,
        {
            title: text [required],
        }
    }
}

/// `probe_notes`' form, validated on a GET so the test needs no CSRF token.
#[form(schema = probe)]
pub struct ProbeForm;

impl RuniqueForm for ProbeForm {
    fn allow_get(&self, _request: &Request) -> bool {
        true
    }
}

/// Validates the form `"probe"` sent in the query, then `save`s it, or
/// `delete`s it with `?delete`. The body tells what came of it.
async fn probe_write(req: Request) -> String {
    let delete = req.query_params.contains_key("delete");
    let form = req.form::<ProbeForm>();
    let form = match ValidationForm::try_form(form, &req, "probe").await {
        Ok(form) => form,
        Err(form) => return format!("invalid: {:?}", form.get_form().errors()),
    };
    let db = &req.engine.db;
    let written = if delete {
        form.delete(db).await
    } else {
        form.save(db).await
    };
    match written {
        Ok(_) => "written".to_string(),
        Err(form) => format!("refused: {:?}", form.get_form().errors()),
    }
}

/// The notes' titles by id, comma-separated.
async fn probe_rows(req: Request) -> String {
    probe::Entity::find()
        .order_by_asc(probe::Column::Id)
        .all(&req.engine.db)
        .await
        .unwrap()
        .into_iter()
        .map(|note| note.title)
        .collect::<Vec<_>>()
        .join(",")
}

async fn my_id(req: Request) -> String {
    format!("{:?}", req.user_id().await)
}

fn probe_routes() -> Routes {
    runique::urlpatterns! {
        "/sign-in/{n}" => get(sign_in), name = "declared_test_probe_sign_in",
        "/my-id" => get(my_id), name = "declared_test_probe_my_id",
        "/probe/rows" => get(probe_rows), name = "declared_test_probe_rows",
        "/open/probe" => get(probe_write), name = "declared_test_probe_open_new",
        "/open/probe/{id}" => get(probe_write), name = "declared_test_probe_open_row",
        admin "/gated/probe" => get(probe_write),
            name = "declared_test_probe_gated_new",
            for_model = probe::Entity,
        admin "/gated/probe/{id}" => get(probe_write),
            name = "declared_test_probe_gated_row",
            for_model = probe::Entity,
    }
}

fn sent(title: &str) -> String {
    format!("runique_form=probe&title={title}")
}

async fn rows(client: &reqwest::Client, base: &str) -> String {
    text(client, format!("{base}/probe/rows")).await
}

/// Each account through the gate: create, update of row 1, delete of row 1.
/// The superuser does everything; the admin through his own grid; the staff
/// member's group only grants updates.
#[tokio::test]
#[serial]
async fn inside_the_constraint_the_gate_decides_by_the_account() {
    for (n, create, update, delete) in [
        (SUPERUSER, true, true, true),
        (ADMIN, true, true, true),
        (STAFF_MEMBER, false, true, false),
    ] {
        let base = spawn_with("", probe_routes()).await;
        let client = signed_in(&base, n).await;

        let body = text(&client, format!("{base}/gated/probe?{}", sent("new"))).await;
        assert_eq!(body == "written", create, "account {n}, create: {body}");
        let expected = if create { "seed,new" } else { "seed" };
        assert_eq!(rows(&client, &base).await, expected, "account {n}");

        let body = text(&client, format!("{base}/gated/probe/1?{}", sent("changed"))).await;
        assert_eq!(body == "written", update, "account {n}, update: {body}");
        let first = if update { "changed" } else { "seed" };
        assert!(rows(&client, &base).await.starts_with(first), "account {n}");

        let body = text(
            &client,
            format!("{base}/gated/probe/1?delete&{}", sent("x")),
        )
        .await;
        assert_eq!(body == "written", delete, "account {n}, delete: {body}");
        assert_eq!(
            !rows(&client, &base).await.starts_with(first),
            delete,
            "account {n}"
        );
    }
}

/// A refusal comes back on the form: the gate's one message, nothing written.
#[tokio::test]
#[serial]
async fn a_refused_write_comes_back_on_the_form() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, STAFF_MEMBER).await;
    let body = text(&client, format!("{base}/gated/probe?{}", sent("new"))).await;
    assert!(body.starts_with("refused:"), "{body}");
    assert_eq!(rows(&client, &base).await, "seed");
}

/// The same handler on a route that isn't declared `admin`: no gate, SeaORM
/// writes, whoever asks — the handler's own check is the developer's.
#[tokio::test]
#[serial]
async fn outside_the_constraint_the_same_handler_writes_without_the_gate() {
    let base = spawn_with("", probe_routes()).await;
    let anonymous = client();
    assert_eq!(
        text(&anonymous, format!("{base}/open/probe?{}", sent("new"))).await,
        "written"
    );
    assert_eq!(
        text(&anonymous, format!("{base}/open/probe/1?{}", sent("changed"))).await,
        "written"
    );
    assert_eq!(rows(&anonymous, &base).await, "changed,new");
    assert_eq!(
        text(&anonymous, format!("{base}/open/probe/1?delete&{}", sent("x"))).await,
        "written"
    );
    assert_eq!(rows(&anonymous, &base).await, "new");
}

/// An account refused at the boundary never reaches the handler: nothing written.
#[tokio::test]
#[serial]
async fn a_refused_account_neither_runs_the_handler_nor_writes() {
    let base = spawn_with("", probe_routes()).await;
    let mut clients = vec![("anonymous", client())];
    for (n, why) in [
        (MEMBER, "no group"),
        (ORPHAN_STAFF_MEMBER, "group whose responsable is no admin"),
        (IDLE_STAFF_MEMBER, "disabled group"),
    ] {
        clients.push((why, signed_in(&base, n).await));
    }
    for (who, client) in &clients {
        let resp = fetch(client, format!("{base}/gated/probe?{}", sent("new")), None).await;
        assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER, "{who}");
        assert_eq!(location(&resp), Some("/"), "{who}");
    }
    assert_eq!(rows(&client(), &base).await, "seed");
}

#[tokio::test]
#[serial]
async fn user_id_reads_the_account_login_put_in_the_session() {
    let base = spawn_with("", probe_routes()).await;
    assert_eq!(text(&client(), format!("{base}/my-id")).await, "None");
    for n in [ADMIN, MEMBER] {
        let client = signed_in(&base, n).await;
        assert_eq!(
            text(&client, format!("{base}/my-id")).await,
            format!("{:?}", Some(pk(n)))
        );
    }
}
