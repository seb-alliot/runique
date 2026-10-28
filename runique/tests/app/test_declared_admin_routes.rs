//! Tests — the routes declared `admin` in `urlpatterns!`: their boundary, which
//! lives at the site's own paths (a refusal must neither show the admin's
//! prefix to a stranger nor close a site session), the `Detail` override, and
//! the forms the macro accepts.

use crate::helpers::db;
use crate::helpers::framework_tables::{
    GROUPES_DDL, GROUPES_DROITS_DDL, HISTORY_DDL, RESET_TOKENS_DDL, USERS_DDL, USERS_GROUPES_DDL,
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

/// Every seeded account's password, for the tests that sign in by a form.
const SIGN_IN_PASSWORD: &str = "a correct horse battery";

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
        RESET_TOKENS_DDL,
        "CREATE TABLE probe_notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT, code TEXT, flag TEXT)",
        "CREATE TABLE bare_notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT)",
    ] {
        db::exec(dbc, ddl).await;
    }
    // Hashed once for the whole file: every test seeds a fresh database.
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let hash = HASH.get_or_init(|| runique::utils::password::hash(SIGN_IN_PASSWORD).expect("hash"));
    let account = |n: u32, name: &str, flags: &str| {
        format!(
            "({}, '{name}', '{name}@example.com', '{hash}', 1, {flags}, '2026-01-01 00:00:00')",
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
    db::exec(
        dbc,
        "INSERT INTO probe_notes (id, title, body, code) VALUES (1, 'seed', 'kept', 'C1')",
    )
    .await;
    db::exec(
        dbc,
        "INSERT INTO bare_notes (id, title, body) VALUES (1, 'bare', 'first')",
    )
    .await;
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
    spawn_with_reset(mount, routes, Some(std::time::Duration::from_secs(3600))).await
}

/// `spawn_with`, the site's reset page lasting `reset_ttl`, or not served.
async fn spawn_with_reset(
    mount: &'static str,
    routes: Routes,
    reset_ttl: Option<std::time::Duration>,
) -> String {
    let dbc = db::fresh_db().await;
    seed(&dbc).await;

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let builder = RuniqueApp::builder(config).with_database(dbc);
    // `with_password_reset` changes the builder's type: the rest of the
    // chain is shared by a macro rather than written twice.
    macro_rules! finish {
        ($builder:expr) => {
            $builder
                .routes(routes)
                .static_files(|s| s.enabled(false))
                .with_admin(|a| {
                    let a =
                        a.path("/secret-admin").site(|s| {
                            s
                                .register(
                                    runique::admin::builder::admin_builder::ModelAdmin::<
                                        probe::Entity,
                                    >::new()
                                    .list(|l| {
                                        l.display([probe::Column::Title])
                                            .filter([probe::Column::Title, probe::Column::Code])
                                            .action(probe::Column::Flag, "OK", "Valider")
                                    })
                                    .form(|f| {
                                        f.fields([
                                            probe::Column::Title,
                                            probe::Column::Code,
                                            probe::Column::Flag,
                                        ])
                                        .readonly([probe::Column::Code])
                                    }),
                                )
                                // No `.form()`: the pages take every field of its own form.
                                .register(runique::admin::builder::admin_builder::ModelAdmin::<
                                    bare::Entity,
                                >::new())
                        });
                    if mount.is_empty() { a } else { a.prefix(mount) }
                })
                .build()
                .await
                .expect("app")
        };
    }
    let app = match reset_ttl {
        Some(ttl) => finish!(builder.with_password_reset(move |pr| pr.token_ttl(ttl))),
        None => finish!(builder),
    };
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
            body: text [nullable],
            code: text [nullable],
            flag: text [nullable],
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

/// Row `id` as `title|body|code|flag`, `-` for a NULL.
async fn probe_row(Path(id): Path<i32>, req: Request) -> String {
    let row = probe::Entity::find_by_id(id)
        .one(&req.engine.db)
        .await
        .unwrap()
        .expect("the row");
    let shown = |value: Option<String>| value.unwrap_or_else(|| "-".into());
    format!(
        "{}|{}|{}|{}",
        row.title,
        shown(row.body),
        shown(row.code),
        shown(row.flag)
    )
}

/// The history rows, `action resource pk summary`, one per line.
async fn probe_history(req: Request) -> String {
    runique::auth::history::Entity::find()
        .all(&req.engine.db)
        .await
        .unwrap()
        .into_iter()
        .map(|h| {
            format!(
                "{} {} {} {}",
                h.action,
                h.resource_key,
                h.object_pk,
                h.summary.unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The accounts holding a reset token, by key, comma-separated and sorted.
async fn probe_reset_token_owners(req: Request) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT user_id FROM eihwaz_reset_tokens ORDER BY user_id",
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|row| {
            row.try_get::<runique::utils::pk::Pk>("", "user_id")
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Account `n`'s groups, by id, comma-separated.
async fn probe_memberships(Path(n): Path<u32>, req: Request) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            format!(
                "SELECT groupe_id FROM eihwaz_users_groupes WHERE user_id = {} ORDER BY groupe_id",
                pk_sql_literal(n)
            ),
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|row| row.try_get::<i32>("", "groupe_id").unwrap().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// The groups, `name:responsable` by name, comma-separated.
async fn probe_groups(req: Request) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT nom, responsable_id FROM eihwaz_groupes ORDER BY nom",
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|row| {
            let nom: String = row.try_get("", "nom").unwrap();
            let responsable: runique::utils::pk::Pk = row.try_get("", "responsable_id").unwrap();
            format!("{nom}:{responsable}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Adds `n` notes, for a list longer than a page.
async fn probe_add_notes(Path(n): Path<u32>, req: Request) -> &'static str {
    use runique::sea_orm::ConnectionTrait;
    for i in 0..n {
        req.engine
            .db
            .execute_unprepared(&format!(
                "INSERT INTO probe_notes (title) VALUES ('added {i}')"
            ))
            .await
            .unwrap();
    }
    "added"
}

/// Blocks account `n` as the staff does: activated once, inactive now.
async fn probe_block(Path(n): Path<u32>, req: Request) -> &'static str {
    use runique::sea_orm::ConnectionTrait;
    req.engine
        .db
        .execute_unprepared(&format!(
            "UPDATE eihwaz_users SET is_active = 0 WHERE id = {}",
            pk_sql_literal(n)
        ))
        .await
        .unwrap();
    "blocked"
}

/// The reset tokens' lifetimes left, in whole minutes, one per line.
async fn probe_reset_tokens(req: Request) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT expires_at FROM eihwaz_reset_tokens",
        ))
        .await
        .unwrap();
    let now = chrono::Utc::now().naive_utc();
    rows.iter()
        .map(|row| {
            let expires: chrono::NaiveDateTime = row.try_get("", "expires_at").unwrap();
            ((expires - now).num_seconds() as f64 / 60.0)
                .round()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Traces a write on an account, as the admin would: the history then holds
/// an entry of a framework table the admin's grid doesn't open.
async fn probe_trace_account(req: Request) -> &'static str {
    runique::auth::history::log_action(
        &req.engine.db,
        runique::auth::history::ActionLog {
            user_id: pk(SUPERUSER),
            username: "carol",
            resource_key: "users",
            object_pk: "1",
            object_repr: "traced-account",
            action: "update",
            summary: Some(r#"{"email":{"old":"alice@example.com","new":"a@example.com"}}"#.into()),
            batch_id: None,
        },
    )
    .await
    .unwrap();
    "ok"
}

async fn my_id(req: Request) -> String {
    format!("{:?}", req.user_id().await)
}

fn probe_routes() -> Routes {
    runique::urlpatterns! {
        "/sign-in/{n}" => get(sign_in), name = "declared_test_probe_sign_in",
        "/my-id" => get(my_id), name = "declared_test_probe_my_id",
        "/flashes" => get(flashes), name = "declared_test_probe_flashes",
        "/probe/rows" => get(probe_rows), name = "declared_test_probe_rows",
        "/probe/row/{id}" => get(probe_row), name = "declared_test_probe_row",
        "/probe/history" => get(probe_history), name = "declared_test_probe_history",
        "/probe/reset-tokens" => get(probe_reset_tokens), name = "declared_test_probe_reset_tokens",
        "/probe/reset-token-owners" => get(probe_reset_token_owners), name = "declared_test_probe_reset_token_owners",
        "/probe/notes/add/{n}" => get(probe_add_notes), name = "declared_test_probe_add_notes",
        "/probe/block/{n}" => get(probe_block), name = "declared_test_probe_block",
        "/probe/groups" => get(probe_groups), name = "declared_test_probe_groups",
        "/probe/memberships/{n}" => get(probe_memberships), name = "declared_test_probe_memberships",
        "/probe/trace-account" => get(probe_trace_account), name = "declared_test_probe_trace_account",
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
        text(
            &anonymous,
            format!("{base}/open/probe/1?{}", sent("changed"))
        )
        .await,
        "written"
    );
    assert_eq!(rows(&anonymous, &base).await, "changed,new");
    assert_eq!(
        text(
            &anonymous,
            format!("{base}/open/probe/1?delete&{}", sent("x"))
        )
        .await,
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

/// The generated pages link to each other by their route names: the list to
/// each row's detail, the detail back to the list, the mount prefix included.
#[tokio::test]
#[serial]
async fn the_list_and_the_detail_link_to_each_other_by_name() {
    for mount in ["", "/mnt"] {
        let base = spawn_with(mount, routes()).await;
        let client = signed_in(&base, SUPERUSER).await;
        let admin = format!("{mount}/secret-admin/probe_notes");

        let list = fetch(&client, format!("{base}{admin}/list"), None).await;
        assert_eq!(list.status(), reqwest::StatusCode::OK, "mount {mount:?}");
        let body = list.text().await.unwrap();
        assert!(
            body.contains(&format!("href=\"{admin}/1/detail")),
            "mount {mount:?}: row link to the detail"
        );

        let detail = fetch(&client, format!("{base}{admin}/1/detail"), None).await;
        assert_eq!(detail.status(), reqwest::StatusCode::OK, "mount {mount:?}");
        let body = detail.text().await.unwrap();
        assert!(
            body.contains(&format!("href=\"{admin}/list\"")),
            "mount {mount:?}: link back to the list"
        );
    }
}

/// The list offers each link on the account's rights: the admin's grid gives
/// create, update and delete; the staff group, update only.
#[tokio::test]
#[serial]
async fn the_list_links_follow_the_rights_of_the_account() {
    let base = spawn_with("", probe_routes()).await;
    let notes = "/secret-admin/probe_notes";

    let admin = signed_in(&base, ADMIN).await;
    let body = text(&admin, format!("{base}{notes}/list")).await;
    for link in [
        format!("href=\"{notes}/1/detail"),
        format!("href=\"{notes}/1/edit\""),
        format!("href=\"{notes}/1/delete\""),
        format!("href=\"{notes}/create\""),
        format!("action=\"{notes}/bulk\""),
    ] {
        assert!(body.contains(&link), "admin: {link}");
    }

    let staff = signed_in(&base, STAFF_MEMBER).await;
    let body = text(&staff, format!("{base}{notes}/list")).await;
    assert!(
        body.contains(&format!("href=\"{notes}/1/edit\"")),
        "staff: edit"
    );
    for link in [
        format!("{notes}/1/delete"),
        format!("{notes}/create"),
        format!("{notes}/bulk"),
    ] {
        assert!(!body.contains(&link), "staff: no {link}");
    }
}

/// The detail page's buttons follow the same rights.
#[tokio::test]
#[serial]
async fn the_detail_buttons_follow_the_rights_of_the_account() {
    let base = spawn_with("", probe_routes()).await;
    let notes = "/secret-admin/probe_notes";

    let admin = signed_in(&base, ADMIN).await;
    let body = text(&admin, format!("{base}{notes}/1/detail")).await;
    assert!(
        body.contains(&format!("href=\"{notes}/1/edit\"")),
        "admin: edit"
    );
    assert!(
        body.contains(&format!("href=\"{notes}/1/delete\"")),
        "admin: delete"
    );

    let staff = signed_in(&base, STAFF_MEMBER).await;
    let body = text(&staff, format!("{base}{notes}/1/detail")).await;
    assert!(
        body.contains(&format!("href=\"{notes}/1/edit\"")),
        "staff: edit"
    );
    assert!(
        !body.contains(&format!("{notes}/1/delete")),
        "staff: no delete"
    );
}

/// The dashboard gets the menu every page gets, the history link included,
/// under the mount prefix.
#[tokio::test]
#[serial]
async fn the_dashboard_has_the_same_menu_as_every_page() {
    for mount in ["", "/mnt"] {
        let base = spawn_with(mount, routes()).await;
        let client = signed_in(&base, SUPERUSER).await;
        let admin = format!("{mount}/secret-admin");
        for page in [format!("{admin}/"), format!("{admin}/probe_notes/list")] {
            let body = text(&client, format!("{base}{page}")).await;
            for link in [
                format!("href=\"{admin}/history\""),
                format!("action=\"{admin}/logout\""),
                format!("href=\"{admin}/probe_notes/list\""),
            ] {
                assert!(body.contains(&link), "{page}: {link}");
            }
        }
    }
}

/// A staff member sees their own groups on the dashboard, with the tables
/// each one opens; an account in no group gets no such card.
#[tokio::test]
#[serial]
async fn the_dashboard_lists_the_accounts_own_groups() {
    let base = spawn().await;
    let staff = text(
        &signed_in(&base, STAFF_MEMBER).await,
        format!("{base}/secret-admin/"),
    )
    .await;
    assert!(staff.contains("data-my-groups"), "{staff}");
    assert!(staff.contains("<strong>relecteurs</strong>"), "{staff}");
    assert!(!staff.contains("<strong>orphelins</strong>"), "{staff}");
    let superuser = text(
        &signed_in(&base, SUPERUSER).await,
        format!("{base}/secret-admin/"),
    )
    .await;
    assert!(!superuser.contains("data-my-groups"), "{superuser}");
}

// ── The edit page ─────────────────────────────────────────────────────────

/// The value of the hidden field `name` in a rendered page: the first
/// `value="…"` after its `name="…"`, whatever lies between.
fn hidden_value(body: &str, name: &str) -> Option<String> {
    let after_name = &body[body.find(&format!("name=\"{name}\""))?..];
    let start = after_name.find("value=\"")? + "value=\"".len();
    let end = after_name[start..].find('"')? + start;
    Some(after_name[start..end].to_string())
}

/// Opens row `id`'s edit page, then sends `fields` with the page's CSRF token.
async fn post_edit(
    client: &reqwest::Client,
    base: &str,
    id: u32,
    fields: &[(&str, &str)],
) -> reqwest::Response {
    let url = format!("{base}/secret-admin/probe_notes/{id}/edit");
    let page = text(client, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").expect("the page carries a CSRF token");
    let name = hidden_value(&page, "runique_form").expect("the page carries the form's name");
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", name.as_str()),
    ];
    body.extend_from_slice(fields);
    client.post(url).form(&body).send().await.unwrap()
}

async fn edit_title(
    client: &reqwest::Client,
    base: &str,
    id: u32,
    title: &str,
) -> reqwest::Response {
    post_edit(client, base, id, &[("title", title)]).await
}

#[tokio::test]
#[serial]
async fn the_edit_page_shows_the_row_and_saves_it_through_the_gate() {
    for n in [SUPERUSER, ADMIN, STAFF_MEMBER] {
        let base = spawn_with("", probe_routes()).await;
        let client = signed_in(&base, n).await;

        let page = fetch(
            &client,
            format!("{base}/secret-admin/probe_notes/1/edit"),
            None,
        )
        .await;
        assert_eq!(page.status(), reqwest::StatusCode::OK, "account {n}");
        let body = page.text().await.unwrap();
        assert!(
            body.contains("value=\"seed\""),
            "account {n}: filled from the row"
        );

        let saved = edit_title(&client, &base, 1, "changed").await;
        assert_eq!(
            saved.status(),
            reqwest::StatusCode::SEE_OTHER,
            "account {n}"
        );
        assert_eq!(
            location(&saved),
            Some("/secret-admin/probe_notes/1/detail"),
            "account {n}"
        );
        assert_eq!(rows(&client, &base).await, "changed", "account {n}");
    }
}

#[tokio::test]
#[serial]
async fn a_post_without_the_csrf_token_writes_nothing() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let resp = client
        .post(format!("{base}/secret-admin/probe_notes/1/edit"))
        .form(&[("runique_form", "admin_update"), ("title", "forged")])
        .send()
        .await
        .unwrap();
    assert_ne!(location(&resp), Some("/secret-admin/probe_notes/1/detail"));
    assert_eq!(rows(&client, &base).await, "seed");
}

/// A row that doesn't exist and one that isn't a key are refused alike, before any form.
#[tokio::test]
#[serial]
async fn a_missing_or_unreadable_row_has_no_edit_page() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    for id in ["99", "abc"] {
        let resp = fetch(
            &client,
            format!("{base}/secret-admin/probe_notes/{id}/edit"),
            None,
        )
        .await;
        assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER, "{id}");
        assert_eq!(location(&resp), Some("/secret-admin/"), "{id}");
    }
}

/// The page shows the declared fields only, the read-only one locked; a POST
/// carrying the others, forged, writes none of them.
#[tokio::test]
#[serial]
async fn an_undeclared_or_readonly_field_is_neither_shown_editable_nor_written() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;

    let page = text(&client, format!("{base}/secret-admin/probe_notes/1/edit")).await;
    assert!(page.contains("name=\"title\""), "declared field shown");
    assert!(!page.contains("name=\"body\""), "undeclared field left out");
    let code = &page[page.find("name=\"code\"").expect("read-only field shown")..];
    assert!(
        code[..code.find('>').unwrap()].contains("readonly"),
        "locked"
    );

    let saved = post_edit(
        &client,
        &base,
        1,
        &[("title", "new"), ("body", "forged"), ("code", "forged")],
    )
    .await;
    assert_eq!(saved.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(
        text(&client, format!("{base}/probe/row/1")).await,
        "new|kept|C1|-"
    );
}

/// An edit leaves one history row, naming the column it changed.
#[tokio::test]
#[serial]
async fn an_edit_leaves_its_history_row() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    assert_eq!(text(&client, format!("{base}/probe/history")).await, "");

    edit_title(&client, &base, 1, "changed").await;
    let history = text(&client, format!("{base}/probe/history")).await;
    let rows: Vec<&str> = history.lines().collect();
    assert_eq!(rows.len(), 1, "{history}");
    assert!(rows[0].starts_with("update probe_notes 1 "), "{history}");
    assert!(rows[0].contains("title"), "{history}");
    assert!(!rows[0].contains("code"), "{history}");
}

// ── The create page ───────────────────────────────────────────────────────

/// Opens the create page, then sends `fields` with the page's CSRF token.
async fn post_create(
    client: &reqwest::Client,
    base: &str,
    fields: &[(&str, &str)],
) -> reqwest::Response {
    let url = format!("{base}/secret-admin/probe_notes/create");
    let page = text(client, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").expect("the page carries a CSRF token");
    let name = hidden_value(&page, "runique_form").expect("the page carries the form's name");
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", name.as_str()),
    ];
    body.extend_from_slice(fields);
    client.post(url).form(&body).send().await.unwrap()
}

/// The superuser and the admin (create ticked on his grid) create a row: the
/// declared, writable field only, then back to the list, with its history row.
#[tokio::test]
#[serial]
async fn the_create_page_writes_a_new_row_through_the_gate() {
    for n in [SUPERUSER, ADMIN] {
        let base = spawn_with("", probe_routes()).await;
        let client = signed_in(&base, n).await;

        let page = text(&client, format!("{base}/secret-admin/probe_notes/create")).await;
        assert!(page.contains("name=\"title\""), "account {n}");
        assert!(
            !page.contains("name=\"body\""),
            "account {n}: undeclared field left out"
        );

        let saved = post_create(
            &client,
            &base,
            &[("title", "new"), ("body", "forged"), ("code", "forged")],
        )
        .await;
        assert_eq!(
            saved.status(),
            reqwest::StatusCode::SEE_OTHER,
            "account {n}"
        );
        assert_eq!(
            location(&saved),
            Some("/secret-admin/probe_notes/list"),
            "account {n}"
        );
        assert_eq!(rows(&client, &base).await, "seed,new", "account {n}");
        assert_eq!(
            text(&client, format!("{base}/probe/row/2")).await,
            "new|-|-|-",
            "account {n}"
        );
        let history = text(&client, format!("{base}/probe/history")).await;
        assert!(
            history.starts_with("create probe_notes 2"),
            "account {n}: {history}"
        );
    }
}

/// The staff group may update, not create: no page, no row, whatever is sent.
#[tokio::test]
#[serial]
async fn without_the_create_right_there_is_no_create_page() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, STAFF_MEMBER).await;
    let page = fetch(
        &client,
        format!("{base}/secret-admin/probe_notes/create"),
        None,
    )
    .await;
    assert_eq!(page.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(location(&page), Some("/secret-admin/"));

    let forged = client
        .post(format!("{base}/secret-admin/probe_notes/create"))
        .form(&[("runique_form", "admin_create"), ("title", "forged")])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&forged), Some("/secret-admin/"));
    assert_eq!(rows(&client, &base).await, "seed");
}

// ── The delete page ───────────────────────────────────────────────────────

/// Opens row `id`'s delete page, then confirms with the page's form only.
async fn confirm_delete(client: &reqwest::Client, base: &str, id: &str) -> reqwest::Response {
    let url = format!("{base}/secret-admin/probe_notes/{id}/delete");
    let page = text(client, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").expect("the page carries a CSRF token");
    let name = hidden_value(&page, "runique_form").expect("the page carries the form's name");
    client
        .post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
        ])
        .send()
        .await
        .unwrap()
}

/// The confirmation sends no field of the model, and still deletes: the form
/// Runique provides for it has none to fill.
#[tokio::test]
#[serial]
async fn the_delete_page_confirms_then_deletes_through_the_gate() {
    for n in [SUPERUSER, ADMIN] {
        let base = spawn_with("", probe_routes()).await;
        let client = signed_in(&base, n).await;

        let page = fetch(
            &client,
            format!("{base}/secret-admin/probe_notes/1/delete"),
            None,
        )
        .await;
        assert_eq!(page.status(), reqwest::StatusCode::OK, "account {n}");
        assert!(
            !page.text().await.unwrap().contains("name=\"title\""),
            "account {n}"
        );

        let deleted = confirm_delete(&client, &base, "1").await;
        assert_eq!(
            deleted.status(),
            reqwest::StatusCode::SEE_OTHER,
            "account {n}"
        );
        assert_eq!(
            location(&deleted),
            Some("/secret-admin/probe_notes/list"),
            "account {n}"
        );
        assert_eq!(rows(&client, &base).await, "", "account {n}");
        let history = text(&client, format!("{base}/probe/history")).await;
        assert!(
            history.starts_with("delete probe_notes 1"),
            "account {n}: {history}"
        );
    }
}

/// The staff group may not delete: no confirmation, and a forged POST deletes nothing.
#[tokio::test]
#[serial]
async fn without_the_delete_right_there_is_no_delete_page() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, STAFF_MEMBER).await;
    let page = fetch(
        &client,
        format!("{base}/secret-admin/probe_notes/1/delete"),
        None,
    )
    .await;
    assert_eq!(location(&page), Some("/secret-admin/"));

    let forged = client
        .post(format!("{base}/secret-admin/probe_notes/1/delete"))
        .form(&[("runique_form", "admin_delete")])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&forged), Some("/secret-admin/"));
    assert_eq!(rows(&client, &base).await, "seed");
}

/// Without the CSRF token the confirmation deletes nothing; a missing or
/// unreadable row has no delete page.
#[tokio::test]
#[serial]
async fn a_delete_needs_the_token_and_an_existing_row() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let forged = client
        .post(format!("{base}/secret-admin/probe_notes/1/delete"))
        .form(&[("runique_form", "admin_delete")])
        .send()
        .await
        .unwrap();
    assert_ne!(location(&forged), Some("/secret-admin/probe_notes/list"));
    assert_eq!(rows(&client, &base).await, "seed");

    for id in ["99", "abc"] {
        let page = fetch(
            &client,
            format!("{base}/secret-admin/probe_notes/{id}/delete"),
            None,
        )
        .await;
        assert_eq!(location(&page), Some("/secret-admin/"), "{id}");
    }
}

/// A model registered without `.form()`: its edit page shows every field of
/// its own form, filled from the row, and saves them through the gate.
#[tokio::test]
#[serial]
async fn without_form_the_edit_page_takes_every_field_of_the_model() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let url = format!("{base}/secret-admin/bare_notes/1/edit");

    let page = text(&client, url.clone()).await;
    assert!(page.contains("value=\"bare\""), "title shown, filled");
    assert!(page.contains("value=\"first\""), "body shown, filled");

    let token = hidden_value(&page, "csrf_token").unwrap();
    let name = hidden_value(&page, "runique_form").unwrap();
    let saved = client
        .post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
            ("title", "bare"),
            ("body", "second"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&saved), Some("/secret-admin/bare_notes/1/detail"));
    let detail = text(&client, format!("{base}/secret-admin/bare_notes/1/detail")).await;
    assert!(detail.contains("second"), "the new body is saved");
}

// ── One sign-in for the admin and the site ────────────────────────────────

/// Signing in on the admin's login page signs in on the whole site: the
/// public pages see the same account.
#[tokio::test]
#[serial]
async fn a_sign_in_on_the_admin_counts_on_the_site_too() {
    for mount in ["", "/mnt"] {
        let base = spawn_with(mount, routes()).await;
        let client = client();
        let login = format!("{base}{mount}/secret-admin/login");
        let page = text(&client, login.clone()).await;
        let token = hidden_value(&page, "csrf_token").expect("CSRF token");
        let resp = client
            .post(login)
            .form(&[
                ("csrf_token", token.as_str()),
                ("username", "carol"),
                ("password", SIGN_IN_PASSWORD),
            ])
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::SEE_OTHER,
            "mount {mount:?}"
        );
        assert_eq!(
            text(&client, format!("{base}/whoami")).await,
            "carol",
            "mount {mount:?}: the site sees the admin's sign-in"
        );
    }
}

// ── The list's filters ────────────────────────────────────────────────────

/// A filter offers the values of the rows in reach, and applies the one picked.
#[tokio::test]
#[serial]
async fn a_filter_offers_the_values_and_narrows_the_list() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let list = format!("{base}/secret-admin/probe_notes/list");

    let page = text(&client, list.clone()).await;
    assert!(
        page.contains("filter_title=seed"),
        "the title's value offered"
    );
    assert!(page.contains("filter_code=C1"), "the code's value offered");

    let narrowed = text(&client, format!("{list}?filter_title=seed")).await;
    assert!(narrowed.contains("#1"), "the matching row stays");
    let empty = text(&client, format!("{list}?filter_title=none")).await;
    assert!(!empty.contains("#1"), "no row matches");
}

/// A column the account can't read is never offered as a filter: its values
/// would show what the row holds.
#[tokio::test]
#[serial]
async fn an_unreadable_column_is_never_offered_as_a_filter() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, ADMIN).await;
    let page = text(&client, format!("{base}/secret-admin/probe_notes/list")).await;
    assert!(
        page.contains("filter_title=seed"),
        "title is ticked on his grid"
    );
    assert!(
        !page.contains("filter_code"),
        "code isn't: neither filter nor value"
    );
    assert!(
        !page.contains("data-filter-col=\"code\""),
        "the code's values never leave the table"
    );
}

// ── The history pages ─────────────────────────────────────────────────────

/// The id of the newest history entry linked from the page.
fn first_entry_link(page: &str) -> Option<String> {
    let start = page.find("/secret-admin/history/")? + "/secret-admin/history/".len();
    let end = page[start..].find('"')? + start;
    Some(page[start..end].to_string())
}

/// An edit shows in the history, and its entry lists the changed column with
/// the value before and after.
#[tokio::test]
#[serial]
async fn the_history_lists_an_edit_and_shows_its_changes() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    edit_title(&client, &base, 1, "changed").await;

    let page = text(&client, format!("{base}/secret-admin/history")).await;
    assert!(
        page.contains(r#"data-action="update""#),
        "the update is listed"
    );
    let id = first_entry_link(&page).expect("a link to the entry");

    let entry = text(&client, format!("{base}/secret-admin/history/{id}")).await;
    // Searched as the diff's cells: every page has a `<title>`, and the
    // row's label "changed" is in it.
    assert!(
        entry.contains(r#"<td class="admin-table__key">title</td>"#),
        "the changed column"
    );
    assert!(
        entry.contains("<td>seed</td>") && entry.contains("<td>changed</td>"),
        "before and after"
    );
}

/// The admin reads `probe_notes` but not its `code` column, and not the
/// accounts: neither shows in his history, whatever he types in the URL.
#[tokio::test]
#[serial]
async fn the_history_shows_only_what_the_account_reads() {
    let base = spawn_with("", probe_routes()).await;
    let superuser = signed_in(&base, SUPERUSER).await;
    // `code` changes through the gated declared route, as the superuser.
    text(
        &superuser,
        format!("{base}/gated/probe/1?{}&code=C2", sent("changed")),
    )
    .await;
    text(&superuser, format!("{base}/probe/trace-account")).await;

    let all = text(&superuser, format!("{base}/secret-admin/history")).await;
    assert!(
        all.contains("traced-account"),
        "the superuser sees the accounts' entry"
    );

    let admin = signed_in(&base, ADMIN).await;
    let page = text(&admin, format!("{base}/secret-admin/history")).await;
    assert!(
        !page.contains("traced-account"),
        "no entry of a table he can't read"
    );
    let id = first_entry_link(&page).expect("the probe_notes entry");
    let entry = text(&admin, format!("{base}/secret-admin/history/{id}")).await;
    assert!(entry.contains("<td>changed</td>"), "the readable column");
    // Searched as the diff's cells: anywhere in the page, a random CSP nonce
    // or CSRF token sometimes holds "C1" or "C2".
    assert!(
        !entry.contains("<td>C2</td>") && !entry.contains("<td>C1</td>"),
        "never the unreadable one"
    );

    // The accounts' entry, typed by hand: refused like a missing one.
    let ids: Vec<u32> = (1..=5).collect();
    for n in ids {
        let resp = fetch(&admin, format!("{base}/secret-admin/history/{n}"), None).await;
        if resp.status() == reqwest::StatusCode::OK {
            assert!(
                !resp.text().await.unwrap().contains("traced-account"),
                "entry {n}"
            );
        }
    }
    let users = text(
        &admin,
        format!("{base}/secret-admin/history?resource=users"),
    )
    .await;
    assert!(
        !users.contains("traced-account"),
        "a filter on a table he can't read is empty"
    );
}

// ── Grouped actions ───────────────────────────────────────────────────────

/// Adds a second note, `second`, through the open declared route.
async fn second_note(client: &reqwest::Client, base: &str) {
    text(client, format!("{base}/open/probe?{}", sent("second"))).await;
}

/// Sends a grouped action from the list's bar, with the page's CSRF token
/// unless `with_token` is false.
async fn bulk(
    client: &reqwest::Client,
    base: &str,
    ids: &str,
    action: &str,
    with_token: bool,
) -> reqwest::Response {
    let list = text(client, format!("{base}/secret-admin/probe_notes/list")).await;
    let token = hidden_value(&list, "csrf_token").unwrap_or_default();
    let mut body = vec![
        ("runique_form", "admin_bulk"),
        ("ids", ids),
        ("bulk_action", action),
        ("return_qs", "page=1&search=sec"),
    ];
    if with_token {
        body.push(("csrf_token", token.as_str()));
    }
    client
        .post(format!("{base}/secret-admin/probe_notes/bulk"))
        .form(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn a_grouped_action_writes_every_checked_row_then_delete_removes_them() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    second_note(&client, &base).await;

    let list = text(&client, format!("{base}/secret-admin/probe_notes/list")).await;
    assert!(
        list.contains("data-bulk-action=\"a0\""),
        "the declared action is offered"
    );
    assert!(
        list.contains("data-bulk-action=\"delete\""),
        "delete is offered"
    );

    let done = bulk(&client, &base, "1,2", "a0", true).await;
    assert_eq!(done.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(
        location(&done),
        Some("/secret-admin/probe_notes/list?page=1&search=sec"),
        "back to the same list page"
    );
    assert_eq!(
        text(&client, format!("{base}/probe/row/1")).await,
        "seed|kept|C1|OK"
    );
    assert!(
        text(&client, format!("{base}/probe/row/2"))
            .await
            .ends_with("|OK")
    );
    let history = text(&client, format!("{base}/probe/history")).await;
    assert_eq!(
        history.matches("update probe_notes").count(),
        2,
        "{history}"
    );

    bulk(&client, &base, "1,2", "delete", true).await;
    assert_eq!(rows(&client, &base).await, "");
}

/// One row refused, nothing written: a missing row among the checked ones
/// stops the delete of the others.
#[tokio::test]
#[serial]
async fn a_grouped_action_is_all_or_nothing() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    bulk(&client, &base, "1,99", "delete", true).await;
    assert_eq!(rows(&client, &base).await, "seed");
    bulk(&client, &base, "1,abc", "a0", true).await;
    assert_eq!(
        text(&client, format!("{base}/probe/row/1")).await,
        "seed|kept|C1|-"
    );
}

/// The admin may update the notes but not their `code`: the action isn't
/// offered, and sent anyway it writes nothing. No delete for the staff group.
#[tokio::test]
#[serial]
async fn a_grouped_action_follows_the_account_rights() {
    let base = spawn_with("", probe_routes()).await;
    let admin = signed_in(&base, ADMIN).await;
    let list = text(&admin, format!("{base}/secret-admin/probe_notes/list")).await;
    assert!(
        !list.contains("data-bulk-action=\"a0\""),
        "flag isn't ticked on his grid"
    );
    bulk(&admin, &base, "1", "a0", true).await;
    assert_eq!(
        text(&admin, format!("{base}/probe/row/1")).await,
        "seed|kept|C1|-"
    );

    let staff = signed_in(&base, STAFF_MEMBER).await;
    let list = text(&staff, format!("{base}/secret-admin/probe_notes/list")).await;
    assert!(
        !list.contains("data-bulk-action=\"delete\""),
        "no delete right"
    );
    bulk(&staff, &base, "1", "delete", true).await;
    assert_eq!(rows(&staff, &base).await, "seed");
}

/// More rows than a page shows, an unknown action, or no CSRF token: nothing.
#[tokio::test]
#[serial]
async fn a_grouped_action_refuses_an_oversized_or_forged_request() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let many = vec!["1"; 500].join(",");
    bulk(&client, &base, &many, "delete", true).await;
    bulk(&client, &base, "1", "a7", true).await;
    bulk(&client, &base, "1", "delete", false).await;
    assert_eq!(rows(&client, &base).await, "seed");
    assert_eq!(
        text(&client, format!("{base}/probe/row/1")).await,
        "seed|kept|C1|-"
    );
}

// ── Resetting an account's password ───────────────────────────────────────

/// The console mailer, for the whole test binary: a reset link needs one.
fn console_mailer() {
    runique::utils::mailer::mailer_init(runique::utils::mailer::MailerConfig {
        backend: runique::utils::mailer::MailerBackend::Console,
        from: "test@runique.rs".to_string(),
        host: String::new(),
        port: 0,
        username: String::new(),
        password: String::new(),
        starttls: false,
    });
}

/// Opens account `n`'s detail, then presses "reset password" (its CSRF
/// token unless `with_token` is false).
async fn press_reset(
    client: &reqwest::Client,
    base: &str,
    n: u32,
    with_token: bool,
) -> reqwest::Response {
    let detail = format!("{base}/secret-admin/users/{}/detail", pk(n));
    let page = text(client, detail).await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let mut body = vec![("runique_form", "admin_reset_password")];
    if with_token {
        body.push(("csrf_token", token.as_str()));
    }
    client
        .post(format!(
            "{base}/secret-admin/users/{}/reset-password",
            pk(n)
        ))
        .form(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn the_superuser_sends_an_account_a_reset_link() {
    console_mailer();
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    let page = text(
        &client,
        format!("{base}/secret-admin/users/{}/detail", pk(MEMBER)),
    )
    .await;
    assert!(page.contains("reset-password"), "the button is offered");

    let resp = press_reset(&client, &base, MEMBER, true).await;
    assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER);
    let detail = format!("/secret-admin/users/{}/detail", pk(MEMBER));
    assert_eq!(location(&resp), Some(detail.as_str()));
    let flashes = text(&client, format!("{base}/flashes")).await;
    assert!(
        flashes.contains("envoy") || flashes.contains("sent"),
        "{flashes}"
    );
    let history = text(
        &client,
        format!("{base}/secret-admin/history?action=reset_password"),
    )
    .await;
    assert!(
        history.contains(r#"data-action="reset_password""#),
        "the reset is traced"
    );
    assert!(history.contains("bob"), "under the account's username");
}

/// The admin's link lasts what the site's reset page is set to, like the
/// site's own links: 15 minutes here, the default hour otherwise.
#[tokio::test]
#[serial]
async fn the_admin_link_lasts_what_the_site_sets() {
    console_mailer();
    for (ttl, minutes) in [(15 * 60, "15"), (3600, "60")] {
        let base = spawn_with_reset(
            "",
            probe_routes(),
            Some(std::time::Duration::from_secs(ttl)),
        )
        .await;
        let client = signed_in(&base, SUPERUSER).await;
        press_reset(&client, &base, MEMBER, true).await;
        assert_eq!(
            text(&client, format!("{base}/probe/reset-tokens")).await,
            minutes
        );
    }
}

/// Without the site's reset page, a link would lead nowhere: none is made.
#[tokio::test]
#[serial]
async fn without_a_reset_page_the_admin_sends_no_link() {
    console_mailer();
    let base = spawn_with_reset("", probe_routes(), None).await;
    let client = signed_in(&base, SUPERUSER).await;
    let resp = press_reset(&client, &base, MEMBER, true).await;
    assert_eq!(resp.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(
        text(&client, format!("{base}/probe/reset-tokens")).await,
        ""
    );
    let history = text(
        &client,
        format!("{base}/secret-admin/history?action=reset_password"),
    )
    .await;
    assert!(
        !history.contains(r#"data-action="reset_password""#),
        "nothing sent, nothing traced"
    );
}

/// The rank rule: an admin never resets a superuser's password; an account
/// that isn't in the accounts' table has none to reset.
#[tokio::test]
#[serial]
async fn a_reset_follows_the_rank_rule_and_the_accounts_table() {
    console_mailer();
    let base = spawn_with("", probe_routes()).await;
    let admin = signed_in(&base, ADMIN).await;
    let resp = admin
        .post(format!(
            "{base}/secret-admin/users/{}/reset-password",
            pk(SUPERUSER)
        ))
        .form(&[("runique_form", "admin_reset_password")])
        .send()
        .await
        .unwrap();
    assert_eq!(
        location(&resp),
        Some("/secret-admin/"),
        "refused before anything"
    );

    let superuser = signed_in(&base, SUPERUSER).await;
    let resp = superuser
        .post(format!("{base}/secret-admin/probe_notes/1/reset-password"))
        .form(&[("runique_form", "admin_reset_password")])
        .send()
        .await
        .unwrap();
    assert_eq!(
        location(&resp),
        Some("/secret-admin/"),
        "no password on a note"
    );
}

/// Without the CSRF token no link is sent, and nothing is traced.
#[tokio::test]
#[serial]
async fn a_reset_needs_the_csrf_token() {
    console_mailer();
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    press_reset(&client, &base, MEMBER, false).await;
    let history = text(
        &client,
        format!("{base}/secret-admin/history?action=reset_password"),
    )
    .await;
    assert!(
        !history.contains(r#"data-action="reset_password""#),
        "nothing traced"
    );
}

/// The account a test reads back: `username|active|activated|has a password`.
async fn account_state(req: Request) -> String {
    let user =
        runique::auth::user::BuiltinUserEntity::find_by_email(&req.engine.db, "newbie@example.com")
            .await;
    match user {
        Some(u) => format!(
            "{}|{}|{}|{}",
            u.username,
            u.is_active,
            u.activated_at.is_some(),
            !u.password.is_empty()
        ),
        None => "none".into(),
    }
}

/// The superuser creates an account from the admin: it is written with a
/// password nobody knows, waits for its owner's activation, and its owner
/// gets the link to choose a password. A password sent with the form is ignored.
#[tokio::test]
#[serial]
async fn the_superuser_creates_an_account_and_its_owner_gets_a_link() {
    console_mailer();
    let routes = probe_routes().merge(runique::urlpatterns! {
        "/probe/account" => get(account_state), name = "declared_test_probe_account",
    });
    let base = spawn_with("", routes).await;
    let client = signed_in(&base, SUPERUSER).await;
    let url = format!("{base}/secret-admin/users/create");
    let page = text(&client, url.clone()).await;
    assert!(
        !page.contains("name=\"password\""),
        "never a password field"
    );
    let token = hidden_value(&page, "csrf_token").unwrap();
    let name = hidden_value(&page, "runique_form").unwrap();
    let resp = client
        .post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
            ("username", "newbie"),
            ("email", "newbie@example.com"),
            ("password", "chosen-by-the-staff"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&resp), Some("/secret-admin/users/list"));
    // Read first: any page built from a `Request` takes the messages.
    let flashes = text(&client, format!("{base}/flashes")).await;
    assert!(
        flashes.contains("ienvenue") || flashes.contains("elcome"),
        "{flashes}"
    );
    // `is_active` is the table's default, not written by the form: false in
    // the framework's migration (with its CHECK), 1 in this test's DDL.
    let state = text(&client, format!("{base}/probe/account")).await;
    assert!(state.starts_with("newbie|"), "{state}");
    assert!(
        state.ends_with("|false|true"),
        "not activated yet, a hashed unknown password: {state}"
    );
}

// ── Ranks ─────────────────────────────────────────────────────────────────

/// Account `n`'s rank, as `Debug` prints it.
async fn probe_rank(Path(n): Path<u32>, req: Request) -> String {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(n))
        .await
        .expect("account");
    format!("{:?}", runique::auth::permissions::rang::Rang::from(&user))
}

fn rank_routes() -> Routes {
    probe_routes().merge(runique::urlpatterns! {
        "/probe/rank/{n}" => get(probe_rank), name = "declared_test_probe_rank",
    })
}

/// Sends "change the rank" of accounts `ids` to `to`, with a CSRF token from
/// the admin's dashboard.
async fn give_rank(
    client: &reqwest::Client,
    base: &str,
    ids: &[u32],
    to: &str,
) -> reqwest::Response {
    let page = text(client, format!("{base}/secret-admin/")).await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let ids = ids
        .iter()
        .map(|n| pk(*n).to_string())
        .collect::<Vec<_>>()
        .join(",");
    client
        .post(format!("{base}/secret-admin/users/rank"))
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", "admin_rank"),
            ("ids", ids.as_str()),
            ("bulk_action", to),
        ])
        .send()
        .await
        .unwrap()
}

async fn rank_of(client: &reqwest::Client, base: &str, n: u32) -> String {
    text(client, format!("{base}/probe/rank/{n}")).await
}

/// The superuser names a dirigeant; with one, he can name admins too. Each
/// change is traced.
#[tokio::test]
#[serial]
async fn the_superuser_names_a_dirigeant_then_admins() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;

    // No dirigeant yet: a second admin would have none above him.
    give_rank(&su, &base, &[MEMBER], "admin").await;
    assert_eq!(rank_of(&su, &base, MEMBER).await, "Lambda");

    give_rank(&su, &base, &[MEMBER], "dirigeant").await;
    assert_eq!(rank_of(&su, &base, MEMBER).await, "Dirigeant");
    give_rank(&su, &base, &[STAFF_MEMBER], "admin").await;
    assert_eq!(rank_of(&su, &base, STAFF_MEMBER).await, "Admin");

    let history = text(&su, format!("{base}/secret-admin/history?action=rank")).await;
    assert_eq!(
        history.matches(r#"data-action="rank""#).count(),
        2,
        "{history}"
    );
}

/// Only a rank strictly above the one given grants it, never on oneself: a
/// dirigeant names no dirigeant, an admin names nobody.
#[tokio::test]
#[serial]
async fn a_rank_is_only_given_from_above() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    give_rank(&su, &base, &[MEMBER], "dirigeant").await;

    give_rank(&su, &base, &[SUPERUSER], "lambda").await;
    assert_eq!(
        rank_of(&su, &base, SUPERUSER).await,
        "Superuser",
        "never oneself"
    );

    let dirigeant = signed_in(&base, MEMBER).await;
    give_rank(&dirigeant, &base, &[STAFF_MEMBER], "dirigeant").await;
    assert_eq!(
        rank_of(&su, &base, STAFF_MEMBER).await,
        "Lambda",
        "not his own rank"
    );
    give_rank(&dirigeant, &base, &[SUPERUSER], "lambda").await;
    assert_eq!(
        rank_of(&su, &base, SUPERUSER).await,
        "Superuser",
        "not above him"
    );

    let admin = signed_in(&base, ADMIN).await;
    give_rank(&admin, &base, &[STAFF_MEMBER], "lambda").await;
    give_rank(&admin, &base, &[ORPHAN_STAFF_MEMBER], "admin").await;
    assert_eq!(
        rank_of(&su, &base, ORPHAN_STAFF_MEMBER).await,
        "Lambda",
        "an admin names nobody"
    );
}

/// The last dirigeant stays while admins remain.
#[tokio::test]
#[serial]
async fn a_rank_change_keeps_the_admins_answered_for() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    give_rank(&su, &base, &[MEMBER], "dirigeant").await;

    // Bob is the only dirigeant, and Alice is still an admin.
    give_rank(&su, &base, &[MEMBER], "lambda").await;
    assert_eq!(rank_of(&su, &base, MEMBER).await, "Dirigeant");
}

/// The active groups' ids, comma-separated.
async fn probe_active_groups(req: Request) -> String {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT id FROM eihwaz_groupes WHERE is_active = 1 ORDER BY id",
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|row| row.try_get::<i32>("", "id").unwrap().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// The groups' list offers to deactivate and activate; an admin does it on
/// the groups he answers for only.
#[tokio::test]
#[serial]
async fn the_groups_are_activated_and_deactivated_from_their_list() {
    let routes = probe_routes().merge(runique::urlpatterns! {
        "/probe/active-groups" => get(probe_active_groups), name = "declared_test_probe_active_groups",
    });
    let base = spawn_with("", routes).await;
    let su = signed_in(&base, SUPERUSER).await;
    save_grid(
        &su,
        &base,
        ADMIN,
        &[
            "t__groupes__read_own",
            "t__groupes__update_own",
            "c__groupes__nom__read",
            "c__groupes__is_active__read",
            "c__groupes__is_active__update",
        ],
    )
    .await;
    let admin = signed_in(&base, ADMIN).await;
    let send = async |ids: &str, action: &str| {
        let list = text(&admin, format!("{base}/secret-admin/groupes/list")).await;
        assert!(list.contains("data-bulk-action=\"a1\""), "{list}");
        let token = hidden_value(&list, "csrf_token").unwrap();
        admin
            .post(format!("{base}/secret-admin/groupes/bulk"))
            .form(&[
                ("csrf_token", token.as_str()),
                ("runique_form", "admin_bulk"),
                ("ids", ids),
                ("bulk_action", action),
            ])
            .send()
            .await
            .unwrap();
    };
    let active = async || text(&admin, format!("{base}/probe/active-groups")).await;

    send("1", "a1").await;
    assert_eq!(active().await, "2", "his group deactivated");
    send("2", "a1").await;
    assert_eq!(active().await, "2", "not another's group");
    send("1,3", "a0").await;
    assert_eq!(active().await, "1,2,3", "his groups activated");
}

/// Any loss of rank hands the account's groups to the actor, taken back
/// and shown waiting for a responsable; an account answering for none just
/// changes rank.
#[tokio::test]
#[serial]
async fn a_demoted_responsable_hands_his_groups_back() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    // Bob answers for "orphelins"; Frank answers for nothing. Carol stays
    // the dirigeants' superuser, Bob the admins' dirigeant until the end.
    give_rank(&su, &base, &[MEMBER, IDLE_STAFF_MEMBER], "dirigeant").await;
    give_rank(&su, &base, &[IDLE_STAFF_MEMBER], "admin").await;
    assert_eq!(rank_of(&su, &base, IDLE_STAFF_MEMBER).await, "Admin");
    let list = format!("{base}/secret-admin/groupes/list");
    assert!(
        !text(&su, list.clone()).await.contains("data-taken-by"),
        "nothing taken back"
    );

    give_rank(&su, &base, &[IDLE_STAFF_MEMBER], "dirigeant").await;
    give_rank(&su, &base, &[MEMBER], "admin").await;
    assert_eq!(rank_of(&su, &base, MEMBER).await, "Admin");
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("orphelins:{}", pk(SUPERUSER))),
        "{groups}"
    );
    assert_eq!(text(&su, list).await.matches("data-taken-by").count(), 1);
}

/// Deleting a responsable from his page hands his groups back first.
#[tokio::test]
#[serial]
async fn the_delete_page_takes_a_responsables_groups_back() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let url = format!("{base}/secret-admin/users/{}/delete", pk(ADMIN));
    let page = text(&su, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").unwrap();
    let name = hidden_value(&page, "runique_form").unwrap();
    su.post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
        ])
        .send()
        .await
        .unwrap();
    let groups = text(&su, format!("{base}/probe/groups")).await;
    for group in ["relecteurs", "en pause"] {
        assert!(
            groups.contains(&format!("{group}:{}", pk(SUPERUSER))),
            "{groups}"
        );
    }
    let history = text(&su, format!("{base}/secret-admin/history?action=transfer")).await;
    assert_eq!(
        history.matches(r#"data-action="transfer""#).count(),
        2,
        "{history}"
    );
}

/// A rank is never a form field: sent with an account's edit, it is ignored.
#[tokio::test]
#[serial]
async fn a_rank_sent_with_an_account_edit_is_ignored() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let url = format!("{base}/secret-admin/users/{}/edit", pk(MEMBER));
    let page = text(&su, url.clone()).await;
    assert!(!page.contains("name=\"is_admin\""), "never a field");
    let token = hidden_value(&page, "csrf_token").unwrap();
    let name = hidden_value(&page, "runique_form").unwrap();
    su.post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
            ("username", "bob"),
            ("email", "bob@example.com"),
            ("is_admin", "true"),
            ("is_dirigeant", "true"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(rank_of(&su, &base, MEMBER).await, "Lambda");
}

// ── A replaced detail, everywhere ─────────────────────────────────────────

/// `routes()` replaces the accounts' detail with `/accounts/{id}`: every
/// link and redirect to an account leads there, never to the generated page
/// (which answers 404).
#[tokio::test]
#[serial]
async fn every_way_to_a_row_follows_its_replaced_detail() {
    console_mailer();
    let base = spawn().await;
    let su = signed_in(&base, SUPERUSER).await;
    let id = pk(MEMBER).to_string();
    let replaced = format!("/accounts/{id}");
    let link = format!("href=\"{replaced}\"");

    // The edit page: its breadcrumb and its cancel button, then its save.
    let edit = format!("{base}/secret-admin/users/{id}/edit");
    let page = text(&su, edit.clone()).await;
    assert!(page.matches(&link).count() >= 2, "breadcrumb and cancel");
    let token = hidden_value(&page, "csrf_token").unwrap();
    let name = hidden_value(&page, "runique_form").unwrap();
    let saved = su
        .post(edit)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", name.as_str()),
            ("username", "bob"),
            ("email", "bob@example.com"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&saved), Some(replaced.as_str()), "after the save");

    // The delete page's cancel button.
    let page = text(&su, format!("{base}/secret-admin/users/{id}/delete")).await;
    assert!(page.contains(&link), "delete's cancel");

    // Back after a reset link and after a rank change.
    let resp = su
        .post(format!("{base}/secret-admin/users/{id}/reset-password"))
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", "admin_reset_password"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&resp), Some(replaced.as_str()), "after a reset");
    let resp = give_rank(&su, &base, &[MEMBER], "dirigeant").await;
    assert_eq!(
        location(&resp),
        Some(replaced.as_str()),
        "after a rank change"
    );
}

// ── Who reaches the admin's login page ────────────────────────────────────

/// Nobody signed in: the login page. An account with admin access: the
/// dashboard. A site account without it: back to `/`, still signed in.
#[tokio::test]
#[serial]
async fn the_login_page_answers_each_visitor_by_their_access() {
    let base = spawn_with("", probe_routes()).await;
    let login = format!("{base}/secret-admin/login");

    let anonymous = client();
    let resp = fetch(&anonymous, login.clone(), None).await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK, "the form");

    let admin = signed_in(&base, ADMIN).await;
    let resp = fetch(&admin, login.clone(), None).await;
    assert_eq!(
        location(&resp),
        Some("/secret-admin/"),
        "already in: the dashboard"
    );

    for n in [MEMBER, ORPHAN_STAFF_MEMBER, IDLE_STAFF_MEMBER] {
        let site = signed_in(&base, n).await;
        let resp = fetch(&site, login.clone(), None).await;
        assert_eq!(location(&resp), Some("/"), "account {n}: back to the site");
        let resp = fetch(&site, format!("{base}/secret-admin/"), None).await;
        assert_eq!(location(&resp), Some("/"), "account {n}: the dashboard too");
        assert_eq!(
            text(&site, format!("{base}/my-id")).await,
            format!("{:?}", Some(pk(n))),
            "account {n}: still signed in on the site"
        );
    }
}

/// Deactivates account `n`, as the staff would.
async fn probe_deactivate(Path(n): Path<u32>, req: Request) -> &'static str {
    use runique::sea_orm::{ActiveModelTrait, Set};
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(n))
        .await
        .expect("account");
    let mut active: runique::auth::user::ActiveModel = user.into();
    active.is_active = Set(false);
    active.update(&req.engine.db).await.unwrap();
    "ok"
}

/// An admin deactivated while signed in: his next admin page closes the
/// session, unlike a site account without admin access.
#[tokio::test]
#[serial]
async fn a_deactivated_admin_loses_the_session_on_his_next_admin_page() {
    let routes = probe_routes().merge(runique::urlpatterns! {
        "/probe/deactivate/{n}" => get(probe_deactivate), name = "declared_test_probe_deactivate",
    });
    let base = spawn_with("", routes).await;
    let admin = signed_in(&base, ADMIN).await;
    text(&client(), format!("{base}/probe/deactivate/{ADMIN}")).await;

    fetch(&admin, format!("{base}/secret-admin/"), None).await;
    assert_eq!(
        text(&admin, format!("{base}/my-id")).await,
        "None",
        "signed out"
    );
}

/// Writes a value in the visitor's session, and reads it back.
async fn probe_session_set(req: Request) -> &'static str {
    req.session.insert("probe_cart", "3 items").await.unwrap();
    "ok"
}

async fn probe_session_get(req: Request) -> String {
    req.session
        .get::<String>("probe_cart")
        .await
        .unwrap()
        .unwrap_or_else(|| "none".into())
}

/// A site visitor who opens the admin's login page keeps their anonymous
/// session: nothing of theirs is the admin's to delete.
#[tokio::test]
#[serial]
async fn an_anonymous_visitor_keeps_their_session_on_the_admin_pages() {
    let routes = probe_routes().merge(runique::urlpatterns! {
        "/probe/session/set" => get(probe_session_set), name = "declared_test_probe_session_set",
        "/probe/session/get" => get(probe_session_get), name = "declared_test_probe_session_get",
    });
    let base = spawn_with("", routes).await;
    let visitor = client();
    text(&visitor, format!("{base}/probe/session/set")).await;
    fetch(&visitor, format!("{base}/secret-admin/login"), None).await;
    fetch(&visitor, format!("{base}/secret-admin/"), None).await;
    assert_eq!(
        text(&visitor, format!("{base}/probe/session/get")).await,
        "3 items"
    );
}

// ── An admin's rights grid ────────────────────────────────────────────────

/// Saves account `n`'s grid with only `boxes` ticked.
async fn save_grid(
    client: &reqwest::Client,
    base: &str,
    n: u32,
    boxes: &[&str],
) -> reqwest::Response {
    let url = format!("{base}/secret-admin/users/{}/rights", pk(n));
    let page = text(client, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let name = hidden_value(&page, "runique_form").unwrap_or_else(|| "admin_rights".into());
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", name.as_str()),
    ];
    for b in boxes {
        body.push((b, "1"));
    }
    client.post(url).form(&body).send().await.unwrap()
}

/// The superuser sees the admin's grid as it is, then narrows it: the admin
/// keeps reading the notes but can no longer edit them.
#[tokio::test]
#[serial]
async fn the_superuser_sets_an_admins_grid_and_it_applies_at_once() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let detail = text(
        &su,
        format!("{base}/secret-admin/users/{}/detail", pk(ADMIN)),
    )
    .await;
    assert!(detail.contains("/rights\""), "the grid is offered");

    let grid = text(
        &su,
        format!("{base}/secret-admin/users/{}/rights", pk(ADMIN)),
    )
    .await;
    assert!(
        grid.contains("name=\"t__probe_notes__update\" value=\"1\" checked"),
        "ticked from the base"
    );
    assert!(!grid.contains("__password__"), "never a secret column");
    assert!(
        !grid.contains("c__users__is_admin__") && !grid.contains("c__users__is_active__"),
        "never a rank nor the activation"
    );

    let saved = save_grid(
        &su,
        &base,
        ADMIN,
        &["t__probe_notes__read", "c__probe_notes__title__read"],
    )
    .await;
    assert_eq!(
        location(&saved),
        Some(format!("/secret-admin/users/{}/detail", pk(ADMIN)).as_str())
    );
    // The trace keeps the grid before and after, not just the tables.
    let history = text(&su, format!("{base}/probe/history")).await;
    let trace = history
        .lines()
        .find(|line| line.starts_with(&format!("rights users {} ", pk(ADMIN))))
        .expect("the grid's trace");
    assert!(
        trace.contains(
            r#""old":"probe_notes (create, read, update, delete; title: read, create, update)""#
        ) && trace.contains(r#""new":"probe_notes (read; title: read)""#),
        "{trace}"
    );

    let admin = signed_in(&base, ADMIN).await;
    let list = fetch(
        &admin,
        format!("{base}/secret-admin/probe_notes/list"),
        None,
    )
    .await;
    assert_eq!(list.status(), reqwest::StatusCode::OK, "still reads");
    let edit = fetch(
        &admin,
        format!("{base}/secret-admin/probe_notes/1/edit"),
        None,
    )
    .await;
    assert_eq!(location(&edit), Some("/secret-admin/"), "no longer edits");
}

/// A box the grid doesn't offer (a table it doesn't know, a secret column)
/// refuses the whole save: nothing changes.
#[tokio::test]
#[serial]
async fn a_forged_grid_is_refused_whole() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    for forged in [
        "t__ghost__read",
        "c__users__password__read",
        "c__users__is_admin__update",
    ] {
        save_grid(&su, &base, ADMIN, &["t__probe_notes__read", forged]).await;
    }
    let admin = signed_in(&base, ADMIN).await;
    let edit = fetch(
        &admin,
        format!("{base}/secret-admin/probe_notes/1/edit"),
        None,
    )
    .await;
    assert_eq!(
        edit.status(),
        reqwest::StatusCode::OK,
        "his grid is untouched"
    );
}

/// Only a dirigeant or the superuser sets a grid, only on an admin, never
/// their own: an admin gets none, a site account has none.
#[tokio::test]
#[serial]
async fn a_grid_is_set_from_above_on_an_admin_only() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let page = fetch(
        &su,
        format!("{base}/secret-admin/users/{}/rights", pk(MEMBER)),
        None,
    )
    .await;
    assert_eq!(
        location(&page),
        Some("/secret-admin/"),
        "a site account has no grid"
    );

    let admin = signed_in(&base, ADMIN).await;
    let page = fetch(
        &admin,
        format!("{base}/secret-admin/users/{}/rights", pk(ADMIN)),
        None,
    )
    .await;
    assert_eq!(location(&page), Some("/secret-admin/"), "never his own");
    save_grid(&admin, &base, ADMIN, &["t__users__delete"]).await;
    let page = fetch(&admin, format!("{base}/secret-admin/users/list"), None).await;
    assert_ne!(
        page.status(),
        reqwest::StatusCode::OK,
        "he gave himself nothing"
    );
}

// ── What the detail and the list offer ────────────────────────────────────

/// The rank form and the reset button show where they apply only: on an
/// account below the viewer, never on the viewer's own page nor on an equal
/// rank, and the reset on the accounts' table alone. The gate decides again
/// on every POST: this is what the page offers.
#[tokio::test]
#[serial]
async fn the_detail_offers_the_rank_and_the_reset_where_they_apply() {
    let base = spawn_with("", rank_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let detail = |n: u32| format!("{base}/secret-admin/users/{}/detail", pk(n));
    let rank_form = r#"action="/secret-admin/users/rank""#;

    let below = text(&su, detail(MEMBER)).await;
    assert!(below.contains(rank_form), "an account below: the rank form");
    assert!(below.contains("/reset-password\""), "and the reset");

    let own = text(&su, detail(SUPERUSER)).await;
    assert!(!own.contains(rank_form), "never on his own page");

    let note = text(&su, format!("{base}/secret-admin/probe_notes/1/detail")).await;
    assert!(
        !note.contains("/reset-password\""),
        "no reset off the accounts"
    );
    assert!(!note.contains("/rank\""), "no rank off the accounts");

    give_rank(&su, &base, &[MEMBER, STAFF_MEMBER], "dirigeant").await;
    let dirigeant = signed_in(&base, MEMBER).await;
    let equal = text(&dirigeant, detail(STAFF_MEMBER)).await;
    assert!(!equal.contains(rank_form), "never on an equal rank");
}

/// On the site's own "forgot password" page, an account the staff blocked
/// gets no link; an active one does.
#[tokio::test]
#[serial]
async fn a_blocked_account_gets_no_link_from_the_site() {
    console_mailer();
    let base = spawn_with("", probe_routes()).await;
    let client = client();
    text(&client, format!("{base}/probe/block/{STAFF_MEMBER}")).await;
    let forgot = |email: &'static str| {
        let client = client.clone();
        let url = format!("{base}/forgot-password");
        async move {
            let page = text(&client, url.clone()).await;
            let token = hidden_value(&page, "csrf_token").unwrap_or_default();
            client
                .post(url)
                .form(&[
                    ("csrf_token", token.as_str()),
                    ("runique_form", "forgot_password"),
                    ("email", email),
                ])
                .send()
                .await
                .unwrap();
        }
    };
    forgot("dave@example.com").await;
    forgot("bob@example.com").await;

    // The links go out in the background: wait for the active account's.
    let owners = format!("{base}/probe/reset-token-owners");
    let mut held = String::new();
    for _ in 0..50 {
        held = text(&client, owners.clone()).await;
        if !held.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(
        held,
        pk(MEMBER).to_string(),
        "bob only, never the blocked dave"
    );
}

/// Previous and next show only where they lead somewhere, and each points
/// to its page; a sorted column's header turns the order around.
#[tokio::test]
#[serial]
async fn the_list_pages_and_sorts_through_its_links() {
    let base = spawn_with("", probe_routes()).await;
    let client = signed_in(&base, SUPERUSER).await;
    // 26 notes, 10 a page: three pages.
    text(&client, format!("{base}/probe/notes/add/25")).await;
    let list = format!("{base}/secret-admin/probe_notes/list");
    let page = |n: u32| text(&client, format!("{list}?page={n}"));
    let no_prev = r#"disabled">&laquo;</span>"#;
    let no_next = r#"disabled">&raquo;</span>"#;

    let first = page(1).await;
    assert!(first.contains(no_prev), "no previous on the first page");
    assert!(first.contains(r#"href="?page=2"#), "next is the second");

    let second = page(2).await;
    assert!(second.contains(r#"href="?page=1"#), "previous is the first");
    assert!(second.contains(r#"href="?page=3"#), "next is the third");

    let last = page(3).await;
    assert!(last.contains(no_next), "no next on the last page");
    assert!(last.contains(r#"href="?page=2"#), "previous is the second");

    let asc = text(&client, format!("{list}?sort_by=title&sort_dir=asc")).await;
    assert!(
        asc.contains("?sort_by=title&sort_dir=desc"),
        "asc turns to desc"
    );
    let desc = text(&client, format!("{list}?sort_by=title&sort_dir=desc")).await;
    assert!(
        desc.contains("?sort_by=title&sort_dir=asc"),
        "desc turns to asc"
    );
}

// ── The groups ────────────────────────────────────────────────────────────

/// The framework serves the groups: the superuser creates one through the
/// generated pages and answers for it, whatever the form sends. An admin
/// without a right on them, and a staff member, never reach them.
#[tokio::test]
#[serial]
async fn the_superuser_creates_a_group_and_answers_for_it() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let url = format!("{base}/secret-admin/groupes/create");
    let page = text(&su, url.clone()).await;
    assert!(!page.contains("name=\"responsable_id\""), "never a field");
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let forged = pk(ADMIN).to_string();
    su.post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", "admin_create"),
            ("nom", "équipe"),
            ("is_active", "true"),
            ("responsable_id", forged.as_str()),
        ])
        .send()
        .await
        .unwrap();
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("équipe:{}", pk(SUPERUSER))),
        "the creator answers for it: {groups}"
    );
    let list = text(&su, format!("{base}/secret-admin/groupes/list")).await;
    assert!(list.contains("équipe"), "listed");

    for n in [ADMIN, STAFF_MEMBER] {
        let client = signed_in(&base, n).await;
        let resp = fetch(&client, format!("{base}/secret-admin/groupes/list"), None).await;
        assert_eq!(location(&resp), Some("/secret-admin/"), "account {n}");
    }
}

/// Saves group `g`'s grid with only `boxes` ticked.
async fn save_group_grid(
    client: &reqwest::Client,
    base: &str,
    g: u32,
    boxes: &[&str],
) -> reqwest::Response {
    let url = format!("{base}/secret-admin/groupes/{g}/rights");
    let page = text(client, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", "admin_rights"),
    ];
    for b in boxes {
        body.push((b, "1"));
    }
    client.post(url).form(&body).send().await.unwrap()
}

/// Whether account `n` reaches row 1's edit page of the notes.
async fn edits_notes(base: &str, n: u32) -> bool {
    let client = signed_in(base, n).await;
    let resp = fetch(
        &client,
        format!("{base}/secret-admin/probe_notes/1/edit"),
        None,
    )
    .await;
    resp.status() == reqwest::StatusCode::OK
}

/// The superuser sets a group's grid from its page: its members get what
/// it says at their next request. The framework's tables are never offered,
/// a forged one refuses the whole save.
#[tokio::test]
#[serial]
async fn a_groups_grid_sets_what_its_members_may_do() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let detail = text(&su, format!("{base}/secret-admin/groupes/1/detail")).await;
    assert!(
        detail.contains("/groupes/1/rights\""),
        "the grid is offered"
    );
    let grid = text(&su, format!("{base}/secret-admin/groupes/1/rights")).await;
    assert!(
        grid.contains("t__probe_notes__read"),
        "the developer's tables"
    );
    assert!(
        !grid.contains("t__users__") && !grid.contains("t__groupes__"),
        "never the framework's"
    );

    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "the group edits the notes"
    );
    save_group_grid(&su, &base, 1, &["t__probe_notes__read", "t__users__read"]).await;
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "a forged box changes nothing"
    );

    save_group_grid(
        &su,
        &base,
        1,
        &["t__probe_notes__read", "c__probe_notes__title__read"],
    )
    .await;
    assert!(!edits_notes(&base, STAFF_MEMBER).await, "reading only, now");
    let list = signed_in(&base, STAFF_MEMBER).await;
    let page = fetch(&list, format!("{base}/secret-admin/probe_notes/list"), None).await;
    assert_eq!(page.status(), reqwest::StatusCode::OK, "still reads");
}

/// An admin sets the grid of a group he answers for only once his own grid
/// lets him update the groups, and never beyond his own rights.
#[tokio::test]
#[serial]
async fn an_admin_sets_his_groups_grid_within_his_own_rights() {
    let base = spawn_with("", probe_routes()).await;
    let admin = signed_in(&base, ADMIN).await;
    let page = fetch(
        &admin,
        format!("{base}/secret-admin/groupes/1/rights"),
        None,
    )
    .await;
    assert_eq!(location(&page), Some("/secret-admin/"), "not delegated");

    let su = signed_in(&base, SUPERUSER).await;
    save_grid(
        &su,
        &base,
        ADMIN,
        &[
            "t__probe_notes__read",
            "t__probe_notes__update",
            "c__probe_notes__title__read",
            "c__probe_notes__title__update",
            "t__groupes__read_own",
            "t__groupes__update_own",
        ],
    )
    .await;
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "his group keeps editing"
    );
    let admin = signed_in(&base, ADMIN).await;
    // Delete is not his: the whole save is refused, the grid stays.
    save_group_grid(
        &admin,
        &base,
        1,
        &["t__probe_notes__read", "t__probe_notes__delete"],
    )
    .await;
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "beyond his rights: refused"
    );

    save_group_grid(
        &admin,
        &base,
        1,
        &["t__probe_notes__read", "c__probe_notes__title__read"],
    )
    .await;
    assert!(!edits_notes(&base, STAFF_MEMBER).await, "within: applied");
}

/// The selector offers the groups the account may set and sends to the one
/// picked; any other id is refused like a missing group. A delegated admin
/// is offered his groups alone, and in their grid only what he may give:
/// neither a table he holds no right on, by name, nor a right he lacks.
#[tokio::test]
#[serial]
async fn the_group_selector_and_grid_show_only_what_the_account_may_set() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let select = |id: &str| format!("{base}/secret-admin/groupes/rights?groupe={id}");
    let page = text(&su, format!("{base}/secret-admin/groupes/1/rights")).await;
    for name in ["relecteurs", "orphelins", "en pause"] {
        assert!(page.contains(&format!(">{name}</option>")), "su: {name}");
    }
    assert_eq!(
        location(&fetch(&su, select("2"), None).await),
        Some("/secret-admin/groupes/2/rights")
    );
    for forged in ["abc", "999"] {
        let resp = fetch(&su, select(forged), None).await;
        assert_eq!(location(&resp), Some("/secret-admin/"), "{forged}");
    }

    save_grid(
        &su,
        &base,
        ADMIN,
        &[
            "t__probe_notes__read",
            "t__probe_notes__update",
            "c__probe_notes__title__read",
            "t__groupes__read_own",
            "t__groupes__update_own",
        ],
    )
    .await;
    let admin = signed_in(&base, ADMIN).await;
    let page = text(&admin, format!("{base}/secret-admin/groupes/1/rights")).await;
    assert!(page.contains(">relecteurs</option>"), "his group");
    assert!(!page.contains(">orphelins</option>"), "never another's");
    assert!(page.contains("t__probe_notes__update"), "a right he holds");
    assert!(
        !page.contains("t__probe_notes__delete"),
        "never one he lacks"
    );
    assert!(!page.contains("bare_notes"), "never a table he can't reach");
    let resp = fetch(&admin, select("2"), None).await;
    assert_eq!(location(&resp), Some("/secret-admin/"), "another's group");
}

/// The groups' list offers a shortcut to their rights, opening the first
/// group the account may set by name; never to one who may set none.
#[tokio::test]
#[serial]
async fn the_groups_list_has_a_shortcut_to_their_rights() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let list = text(&su, format!("{base}/secret-admin/groupes/list")).await;
    let shortcut = r#"href="/secret-admin/groupes/rights""#;
    assert!(list.contains(shortcut), "offered");
    let resp = fetch(&su, format!("{base}/secret-admin/groupes/rights"), None).await;
    assert_eq!(
        location(&resp),
        Some("/secret-admin/groupes/3/rights"),
        "the first by name: en pause"
    );

    // An admin with a read right on the groups but no delegation to set them.
    save_grid(&su, &base, ADMIN, &["t__groupes__read_own"]).await;
    let admin = signed_in(&base, ADMIN).await;
    let list = text(&admin, format!("{base}/secret-admin/groupes/list")).await;
    assert!(!list.contains(shortcut), "not to one who sets none");
}

/// On an admin's grid, the groups offer their floor alone (create, and act
/// on his own); a full right forged in the save refuses it whole.
#[tokio::test]
#[serial]
async fn an_admins_grid_keeps_the_groups_floor() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let grid = text(
        &su,
        format!("{base}/secret-admin/users/{}/rights", pk(ADMIN)),
    )
    .await;
    for floor in ["create", "read_own", "update_own", "delete_own"] {
        let name = format!(r#"name="t__groupes__{floor}""#);
        assert!(grid.contains(&name), "offered: {floor}");
    }
    for full in ["read", "update", "delete"] {
        let name = format!(r#"name="t__groupes__{full}""#);
        assert!(!grid.contains(&name), "never offered: {full}");
    }

    save_grid(&su, &base, ADMIN, &["t__groupes__read"]).await;
    assert!(
        edits_notes(&base, ADMIN).await,
        "a forged full right refused the save: his grid stays"
    );
}

/// Deleting a group takes its memberships and rights with it: its members
/// lose what it gave at their next request.
#[tokio::test]
#[serial]
async fn deleting_a_group_takes_its_members_rights() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "the group gives the edit"
    );

    let url = format!("{base}/secret-admin/groupes/1/delete");
    let page = text(&su, url.clone()).await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    su.post(url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", "admin_delete"),
        ])
        .send()
        .await
        .unwrap();

    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(!groups.contains("relecteurs"), "deleted: {groups}");
    let staff = signed_in(&base, STAFF_MEMBER).await;
    let resp = fetch(
        &staff,
        format!("{base}/secret-admin/probe_notes/list"),
        None,
    )
    .await;
    assert_eq!(location(&resp), Some("/"), "no group left: no admin access");
}

/// Sends account `n`'s groups form with only `groups` ticked.
async fn save_memberships(
    client: &reqwest::Client,
    base: &str,
    n: u32,
    groups: &[&str],
) -> reqwest::Response {
    let page = text(
        client,
        format!("{base}/secret-admin/users/{}/detail", pk(n)),
    )
    .await;
    let token = hidden_value(&page, "csrf_token").unwrap_or_default();
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", "admin_groups"),
    ];
    for g in groups {
        body.push((g, "1"));
    }
    client
        .post(format!("{base}/secret-admin/users/{}/groups", pk(n)))
        .form(&body)
        .send()
        .await
        .unwrap()
}

/// The superuser puts an account in a group from its page: it gets the
/// group's rights at once, and loses them when taken out. Never on his own
/// page; a group an admin answers for takes no account with a rank.
#[tokio::test]
#[serial]
async fn an_account_joins_and_leaves_a_group_from_its_page() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    let page = text(
        &su,
        format!("{base}/secret-admin/users/{}/detail", pk(MEMBER)),
    )
    .await;
    assert!(page.contains(r#"name="g__1""#), "the groups are offered");
    let own = text(
        &su,
        format!("{base}/secret-admin/users/{}/detail", pk(SUPERUSER)),
    )
    .await;
    assert!(!own.contains("g__1"), "never on his own page");

    assert!(!edits_notes(&base, MEMBER).await, "no group yet");
    save_memberships(&su, &base, MEMBER, &["g__1"]).await;
    assert_eq!(
        text(&su, format!("{base}/probe/memberships/{MEMBER}")).await,
        "1"
    );
    assert!(
        edits_notes(&base, MEMBER).await,
        "the group's rights, at once"
    );

    save_memberships(&su, &base, MEMBER, &[]).await;
    assert_eq!(
        text(&su, format!("{base}/probe/memberships/{MEMBER}")).await,
        ""
    );
    assert!(!edits_notes(&base, MEMBER).await, "taken out: gone");

    // The admin answers for group 1: it takes site accounts only.
    save_memberships(&su, &base, ADMIN, &["g__1"]).await;
    assert_eq!(
        text(&su, format!("{base}/probe/memberships/{ADMIN}")).await,
        ""
    );
}

/// A delegated admin sees and sets only his groups; the account's other
/// memberships stay, and a group of another's forged in the form refuses
/// the whole save.
#[tokio::test]
#[serial]
async fn an_admin_sets_only_his_own_groups_on_an_account() {
    let base = spawn_with("", probe_routes()).await;
    let su = signed_in(&base, SUPERUSER).await;
    save_memberships(&su, &base, MEMBER, &["g__1", "g__2"]).await;
    save_grid(
        &su,
        &base,
        ADMIN,
        &[
            "t__users__read",
            "c__users__username__read",
            "t__groupes__read_own",
            "t__groupes__update_own",
        ],
    )
    .await;

    let admin = signed_in(&base, ADMIN).await;
    let page = text(
        &admin,
        format!("{base}/secret-admin/users/{}/detail", pk(MEMBER)),
    )
    .await;
    assert!(page.contains(r#"name="g__1""#), "his group");
    assert!(!page.contains(r#"name="g__2""#), "never another's");

    // Frank belongs to group 3 alone: group 2, another's, forged for him.
    save_memberships(&admin, &base, IDLE_STAFF_MEMBER, &["g__3", "g__2"]).await;
    assert_eq!(
        text(&su, format!("{base}/probe/memberships/{IDLE_STAFF_MEMBER}")).await,
        "3",
        "forged: nothing changed"
    );
    save_memberships(&admin, &base, MEMBER, &[]).await;
    assert_eq!(
        text(&su, format!("{base}/probe/memberships/{MEMBER}")).await,
        "2",
        "out of his group, still in the other"
    );
}

// ── Handing a group over ──────────────────────────────────────────────────

/// Marks group `g` as taken back by the superuser, as a departure would.
async fn probe_take_back(Path(g): Path<i32>, req: Request) -> &'static str {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let db = &req.engine.db;
    db.execute_raw(Statement::from_string(
        db.get_database_backend(),
        format!(
            "UPDATE eihwaz_groupes SET repris_par = {} WHERE id = {g}",
            pk_sql_literal(SUPERUSER)
        ),
    ))
    .await
    .unwrap();
    "ok"
}

fn transfer_routes() -> Routes {
    rank_routes().merge(runique::urlpatterns! {
        "/probe/take-back/{g}" => get(probe_take_back), name = "declared_test_probe_take_back",
    })
}

/// Bob a dirigeant, Eve an admin with an empty grid: two receivers.
async fn with_receivers(base: &str) -> reqwest::Client {
    let su = signed_in(base, SUPERUSER).await;
    give_rank(&su, base, &[MEMBER], "dirigeant").await;
    give_rank(&su, base, &[ORPHAN_STAFF_MEMBER], "admin").await;
    su
}

fn transfer_page(base: &str, g: i32, to: u32) -> String {
    format!("{base}/secret-admin/groupes/{g}/transfer?to={}", pk(to))
}

/// Confirms the transfer of group `g` to `to`, sending back the losses the
/// page showed, or `shown` instead.
async fn confirm_transfer(
    client: &reqwest::Client,
    base: &str,
    g: i32,
    to: u32,
    shown: Option<&str>,
    keep: &[&str],
) -> reqwest::Response {
    let page = text(client, transfer_page(base, g, to)).await;
    // From the dashboard: a refused transfer page redirects, with no form.
    let dashboard = text(client, format!("{base}/secret-admin/")).await;
    let token = hidden_value(&dashboard, "csrf_token").unwrap_or_default();
    let sent = shown.map_or_else(
        || hidden_value(&page, "shown").unwrap_or_default(),
        str::to_string,
    );
    let to = pk(to).to_string();
    let mut body = vec![
        ("csrf_token", token.as_str()),
        ("runique_form", "admin_transfer"),
        ("to", to.as_str()),
        ("shown", sent.as_str()),
    ];
    body.extend(keep.iter().map(|name| (*name, "1")));
    client
        .post(format!("{base}/secret-admin/groupes/{g}/transfer"))
        .form(&body)
        .send()
        .await
        .unwrap()
}

/// Handed to an admin, a group keeps only what his grid covers: the page
/// lists the losses first, the transfer stores the reduction and is traced.
#[tokio::test]
#[serial]
async fn a_group_handed_to_an_admin_is_reduced_to_his_rights() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let detail = text(&su, format!("{base}/secret-admin/groupes/1/detail")).await;
    assert!(
        detail.contains("/groupes/1/transfer\""),
        "offered: {detail}"
    );
    assert!(
        detail.contains(&format!("value=\"{}\"", pk(ORPHAN_STAFF_MEMBER))),
        "eve offered"
    );
    assert!(
        !detail.contains(&format!("value=\"{}\"", pk(STAFF_MEMBER))),
        "a lambda isn't"
    );

    let page = text(&su, transfer_page(&base, 1, ORPHAN_STAFF_MEMBER)).await;
    assert!(
        page.contains("data-lines") && page.contains("name=\"keep__0\""),
        "{page}"
    );
    assert!(
        !page.contains("checked"),
        "a sideways transfer keeps nothing at first"
    );
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "nothing written by the page"
    );

    let done = confirm_transfer(&su, &base, 1, ORPHAN_STAFF_MEMBER, None, &[]).await;
    assert_eq!(location(&done), Some("/secret-admin/groupes/1/detail"));
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("relecteurs:{}", pk(ORPHAN_STAFF_MEMBER))),
        "{groups}"
    );
    assert!(
        !edits_notes(&base, STAFF_MEMBER).await,
        "the group lost the notes"
    );
    let history = text(&su, format!("{base}/secret-admin/history?action=transfer")).await;
    assert_eq!(
        history.matches(r#"data-action="transfer""#).count(),
        1,
        "{history}"
    );
}

/// Handed to a dirigeant, a group stays whole: no loss to confirm.
#[tokio::test]
#[serial]
async fn a_group_handed_to_a_dirigeant_stays_whole() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let page = text(&su, transfer_page(&base, 1, MEMBER)).await;
    assert!(page.contains("data-no-loss"), "{page}");
    confirm_transfer(&su, &base, 1, MEMBER, None, &[]).await;
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("relecteurs:{}", pk(MEMBER))),
        "{groups}"
    );
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "the group keeps its rights"
    );
}

/// Losses other than those shown write nothing and show the page again.
#[tokio::test]
#[serial]
async fn a_transfer_goes_through_only_for_the_losses_shown() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let resp = confirm_transfer(&su, &base, 1, ORPHAN_STAFF_MEMBER, Some(""), &[]).await;
    assert!(
        location(&resp).is_some_and(|l| l.contains("/groupes/1/transfer?to=")),
        "shown again"
    );
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("relecteurs:{}", pk(ADMIN))),
        "{groups}"
    );
    assert!(edits_notes(&base, STAFF_MEMBER).await);
}

/// Refused: by an admin, to a lambda, to the responsable already in place,
/// and to an admin when the group has a member with a rank.
#[tokio::test]
#[serial]
async fn a_transfer_is_refused_outside_its_rules() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let before = text(&su, format!("{base}/probe/groups")).await;

    let admin = signed_in(&base, ADMIN).await;
    let detail = text(&admin, format!("{base}/secret-admin/groupes/1/detail")).await;
    assert!(!detail.contains("/transfer\""), "not offered to an admin");
    confirm_transfer(&admin, &base, 1, MEMBER, Some(""), &[]).await;
    confirm_transfer(&su, &base, 1, STAFF_MEMBER, Some(""), &[]).await;
    confirm_transfer(&su, &base, 1, ADMIN, Some(""), &[]).await;
    // "orphelins" counts Eve, now an admin.
    confirm_transfer(&su, &base, 2, ADMIN, Some(""), &[]).await;
    let flashes = text(&su, format!("{base}/flashes")).await;
    assert!(
        flashes.contains(&*runique::utils::trad::t("admin.transfer.ranked_members")),
        "{flashes}"
    );
    assert_eq!(text(&su, format!("{base}/probe/groups")).await, before);
}

/// A group taken back shows who took it, on its list and its page, until
/// it is handed over.
#[tokio::test]
#[serial]
async fn a_group_taken_back_waits_for_a_responsable() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let list = format!("{base}/secret-admin/groupes/list");
    assert!(!text(&su, list.clone()).await.contains("data-taken-by"));
    text(&su, format!("{base}/probe/take-back/1")).await;
    for page in [
        list.clone(),
        format!("{base}/secret-admin/groupes/1/detail"),
    ] {
        let body = text(&su, page.clone()).await;
        assert!(
            body.contains("data-taken-by") && body.contains("carol"),
            "{page}"
        );
    }
    confirm_transfer(&su, &base, 1, MEMBER, None, &[]).await;
    assert!(
        !text(&su, list).await.contains("data-taken-by"),
        "handed over"
    );
}

/// A right kept goes to the receiver's grid and stays with the group; a
/// box the page didn't offer refuses the whole.
#[tokio::test]
#[serial]
async fn a_right_kept_goes_to_the_receiver_and_stays_with_the_group() {
    let base = spawn_with("", transfer_routes()).await;
    let su = with_receivers(&base).await;
    let forged = confirm_transfer(&su, &base, 1, ORPHAN_STAFF_MEMBER, None, &["keep__99"]).await;
    assert_eq!(location(&forged), Some("/secret-admin/groupes/1/detail"));
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("relecteurs:{}", pk(ADMIN))),
        "nothing written: {groups}"
    );

    let page = text(&su, transfer_page(&base, 1, ORPHAN_STAFF_MEMBER)).await;
    let every: Vec<String> = (0..)
        .map(|i| format!("keep__{i}"))
        .take_while(|name| page.contains(&format!("name=\"{name}\"")))
        .collect();
    assert!(!every.is_empty(), "{page}");
    let keep: Vec<&str> = every.iter().map(String::as_str).collect();
    confirm_transfer(&su, &base, 1, ORPHAN_STAFF_MEMBER, None, &keep).await;
    let groups = text(&su, format!("{base}/probe/groups")).await;
    assert!(
        groups.contains(&format!("relecteurs:{}", pk(ORPHAN_STAFF_MEMBER))),
        "{groups}"
    );
    assert!(
        edits_notes(&base, STAFF_MEMBER).await,
        "every right kept: the group edits still"
    );
    assert!(
        edits_notes(&base, ORPHAN_STAFF_MEMBER).await,
        "the receiver got them"
    );
}
