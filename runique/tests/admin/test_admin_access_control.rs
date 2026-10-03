//! Admin access control with a real staff member, not a superuser — written
//! from cargo-mutants survivors (2026-10-02): every other admin test logs in as
//! a superuser, who passes every check, so the denial path was never exercised.
//! "editor" belongs to the seeded "Éditeurs" group: read-only on `groupes`,
//! nothing on `users`.
use crate::helpers::admin_server::{self, ADMIN_PREFIX, SEED_GROUPE_ID};
use crate::helpers::pk::pk_sql_literal;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use serial_test::serial;

const EDITOR: &str = "editor";
const EDITOR_PASSWORD: &str = "editor_password_123";

async fn spawn() -> (String, DatabaseConnection) {
    let (router, dbc) = admin_server::build_admin_app().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (format!("http://{addr}"), dbc)
}

async fn add_editor(db: &DatabaseConnection) {
    let hash = runique::utils::password::hash(EDITOR_PASSWORD).unwrap();
    let id = pk_sql_literal(50);
    db.execute_unprepared(&format!(
        "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
         VALUES ({id}, '{EDITOR}', 'editor@example.com', '{hash}', 1, 1, 0, '2026-01-01 00:00:00')"
    ))
    .await
    .unwrap();
    db.execute_unprepared(&format!(
        "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({id}, {SEED_GROUPE_ID})"
    ))
    .await
    .unwrap();
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn csrf(client: &reqwest::Client, url: &str) -> String {
    let resp = client.get(url).send().await.unwrap();
    resp.headers()["x-csrf-token"].to_str().unwrap().to_string()
}

async fn login_editor(base: &str) -> reqwest::Client {
    let client = client();
    let url = format!("{base}{ADMIN_PREFIX}/login");
    let token = csrf(&client, &url).await;
    let resp = client
        .post(&url)
        .form(&[
            ("username", EDITOR),
            ("password", EDITOR_PASSWORD),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection(), "editor logs in");
    client
}

async fn groupe_count(db: &DatabaseConnection) -> i64 {
    crate::helpers::db::count(db, "eihwaz_groupes").await
}

#[tokio::test]
#[serial]
async fn a_staff_member_gets_only_what_their_group_grants() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let client = login_editor(&base).await;

    // Read on `groupes`: granted.
    let list = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/list"))
        .send()
        .await
        .unwrap();
    assert_eq!(list.status(), 200);
    assert!(list.text().await.unwrap().contains("Éditeurs"));

    // Nothing on `users`: turned away, without the list.
    let users = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .unwrap();
    assert!(users.status().is_redirection(), "users: {}", users.status());
    let to = users.headers()["location"].to_str().unwrap().to_string();
    assert!(!to.contains("/users"), "sent away from users, got {to}");

    // Read-only on `groupes`: no create form.
    let create = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/create"))
        .send()
        .await
        .unwrap();
    assert!(create.status().is_redirection());
    assert_eq!(
        create.headers()["location"].to_str().unwrap(),
        format!("{ADMIN_PREFIX}/groupes/list")
    );
}

#[tokio::test]
#[serial]
async fn a_read_only_staff_member_cannot_write_by_posting_directly() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let client = login_editor(&base).await;
    let before = groupe_count(&db).await;
    let list_url = format!("{base}{ADMIN_PREFIX}/groupes/list");

    for (path, fields) in [
        ("create".to_string(), vec![("nom", "Hackers")]),
        (format!("{SEED_GROUPE_ID}/edit"), vec![("nom", "Pwned")]),
        (format!("{SEED_GROUPE_ID}/delete"), vec![]),
    ] {
        let token = csrf(&client, &list_url).await;
        let mut form: Vec<(&str, &str)> = fields.clone();
        form.push(("csrf_token", &token));
        let resp = client
            .post(format!("{base}{ADMIN_PREFIX}/groupes/{path}"))
            .form(&form)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_redirection(), "{path}: {}", resp.status());
    }

    assert_eq!(
        groupe_count(&db).await,
        before,
        "nothing created or deleted"
    );
    let name: String = db
        .query_one_raw(runique::sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!("SELECT nom FROM eihwaz_groupes WHERE id = {SEED_GROUPE_ID}"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "nom")
        .unwrap();
    assert_eq!(name, "Éditeurs", "nothing edited");
}

async fn login_superuser(base: &str) -> reqwest::Client {
    use crate::helpers::admin_server::{SUPERUSER_PASSWORD, SUPERUSER_USERNAME};
    let client = client();
    let url = format!("{base}{ADMIN_PREFIX}/login");
    let token = csrf(&client, &url).await;
    let resp = client
        .post(&url)
        .form(&[
            ("username", SUPERUSER_USERNAME),
            ("password", SUPERUSER_PASSWORD),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection());
    client
}

async fn droits_of(db: &DatabaseConnection, groupe: i64) -> i64 {
    db.query_one_raw(runique::sea_orm::Statement::from_string(
        db.get_database_backend(),
        format!("SELECT COUNT(*) AS n FROM eihwaz_groupes_droits WHERE groupe_id = {groupe}"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

/// A nested POST reaches a child only under its own parent — written from
/// cargo-mutants survivors (2026-10-02): no test posted through a nested route.
#[tokio::test]
#[serial]
async fn a_nested_post_acts_only_under_the_childs_own_parent() {
    use crate::helpers::admin_server::SEED_DROIT_RESOURCE_KEY;
    let (base, db) = spawn().await;
    let client = login_superuser(&base).await;
    let list = format!("{base}{ADMIN_PREFIX}/groupes/list");
    db.execute_unprepared("INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'Autres')")
        .await
        .unwrap();

    let delete = |parent: i64| {
        format!("{base}{ADMIN_PREFIX}/groupes/{parent}/droits/{SEED_DROIT_RESOURCE_KEY}/delete")
    };

    let token = csrf(&client, &list).await;
    let _ = client
        .post(delete(2))
        .form(&[("csrf_token", &token)])
        .send()
        .await
        .unwrap();
    assert_eq!(
        droits_of(&db, SEED_GROUPE_ID).await,
        1,
        "group 2's route can't reach group 1's right"
    );

    let token = csrf(&client, &list).await;
    let resp = client
        .post(delete(SEED_GROUPE_ID))
        .form(&[("csrf_token", &token)])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection(), "{}", resp.status());
    assert_eq!(
        droits_of(&db, SEED_GROUPE_ID).await,
        0,
        "deleted under its own parent"
    );
}

/// Empty list parameters (an untouched search box, a cleared filter) mean "no
/// constraint", not "match the empty string".
#[tokio::test]
#[serial]
async fn empty_list_parameters_are_ignored() {
    let (base, _db) = spawn().await;
    let client = login_superuser(&base).await;
    for query in [
        "filter_nom=",
        "search=",
        "sort_by=",
        "search=&sort_by=&filter_nom=",
    ] {
        let resp = client
            .get(format!("{base}{ADMIN_PREFIX}/groupes/list?{query}"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "{query}");
        assert!(resp.text().await.unwrap().contains("Éditeurs"), "{query}");
    }
}

async fn memberships(db: &DatabaseConnection) -> Vec<i64> {
    db.query_all_raw(runique::sea_orm::Statement::from_string(
        db.get_database_backend(),
        "SELECT groupe_id FROM eihwaz_users_groupes ORDER BY groupe_id".to_string(),
    ))
    .await
    .unwrap()
    .iter()
    .map(|row| row.try_get::<i64>("", "groupe_id").unwrap())
    .collect()
}

async fn edit_editor(
    client: &reqwest::Client,
    base: &str,
    groupes: &[&str],
) -> reqwest::StatusCode {
    let url = format!(
        "{base}{ADMIN_PREFIX}/users/{}/edit",
        crate::helpers::pk::pk(50)
    );
    let token = csrf(client, &url).await;
    let mut form = vec![
        ("username", "editor"),
        ("email", "editor@example.com"),
        ("is_staff", "true"),
        ("is_active", "true"),
        ("csrf_token", token.as_str()),
    ];
    form.extend(groupes.iter().map(|g| ("groupes", *g)));
    client.post(&url).form(&form).send().await.unwrap().status()
}

/// Group changes made on a user's edit page are saved — written while
/// checking cargo-mutants survivors (2026-10-03): the edit went through the
/// partial update, which ignored `groupes`, so unchecking a privileged group
/// reported success and left the user in it.
#[tokio::test]
#[serial]
async fn editing_a_user_saves_their_groups() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    db.execute_unprepared("INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'Relecteurs')")
        .await
        .unwrap();
    let client = login_superuser(&base).await;

    assert!(
        edit_editor(&client, &base, &["1", "2"])
            .await
            .is_redirection()
    );
    assert_eq!(memberships(&db).await, [1, 2], "a checked group is added");

    assert!(edit_editor(&client, &base, &[]).await.is_redirection());
    assert!(
        memberships(&db).await.is_empty(),
        "unchecked groups are removed"
    );
}

async fn rights(db: &DatabaseConnection) -> Vec<(i64, String, bool)> {
    db.query_all_raw(runique::sea_orm::Statement::from_string(
        db.get_database_backend(),
        "SELECT groupe_id, resource_key, can_delete FROM eihwaz_groupes_droits ORDER BY groupe_id, resource_key"
            .to_string(),
    ))
    .await
    .unwrap()
    .iter()
    .map(|r| {
        (
            r.try_get::<i64>("", "groupe_id").unwrap(),
            r.try_get::<String>("", "resource_key").unwrap(),
            r.try_get::<bool>("", "can_delete").unwrap(),
        )
    })
    .collect()
}

/// Under a nested route the parent comes from the URL, never from the body:
/// a tampered `groupe_id` (or, on edit, `resource_key`) can't move the row to
/// another parent — written from cargo-mutants survivors (2026-10-03).
#[tokio::test]
#[serial]
async fn a_nested_form_cannot_be_moved_to_another_parent() {
    let (base, db) = spawn().await;
    db.execute_unprepared("INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'Autres')")
        .await
        .unwrap();
    let client = login_superuser(&base).await;
    let nested = format!("{base}{ADMIN_PREFIX}/groupes/{SEED_GROUPE_ID}/droits");

    // The form shows the parent as a fixed hidden value, not a picker.
    let page = client
        .get(format!("{nested}/create"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let at = page.find("name=\"groupe_id\"").expect("groupe_id field");
    let tag = &page[page[..at].rfind('<').unwrap()..at + page[at..].find('>').unwrap()];
    assert!(
        tag.contains("hidden") && tag.contains(&format!("value=\"{SEED_GROUPE_ID}\"")),
        "{tag}"
    );

    let token = csrf(&client, &format!("{nested}/create")).await;
    let resp = client
        .post(format!("{nested}/create"))
        .form(&[
            ("groupe_id", "2"),
            ("resource_key", "users"),
            ("can_read", "true"),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection(), "{}", resp.status());

    let token = csrf(&client, &format!("{nested}/create")).await;
    let resp = client
        .post(format!("{nested}/users/edit"))
        .form(&[
            ("groupe_id", "2"),
            ("resource_key", "groupes"),
            ("can_read", "true"),
            ("can_delete", "true"),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection(), "{}", resp.status());

    assert_eq!(
        rights(&db).await,
        [
            (SEED_GROUPE_ID, "groupes".to_string(), false),
            (SEED_GROUPE_ID, "users".to_string(), true),
        ],
        "created and edited under group 1, keys pinned"
    );
}

fn hidden_value(page: &str, name: &str) -> Option<String> {
    let at = page.find(&format!("name=\"{name}\""))?;
    let rest = &page[at..];
    let v = rest.find("value=\"")? + 7;
    Some(rest[v..v + rest[v..].find('"')?].to_string())
}

/// A duplicate is shown on the form, not turned into an error page — on create
/// and on edit — written from cargo-mutants survivors (2026-10-03).
#[tokio::test]
#[serial]
async fn a_duplicate_is_reported_on_the_form() {
    use crate::helpers::admin_server::SEED_DROIT_RESOURCE_KEY;
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let client = login_superuser(&base).await;

    let create = format!("{base}{ADMIN_PREFIX}/groupes/{SEED_GROUPE_ID}/droits/create");
    let token = csrf(&client, &create).await;
    let resp = client
        .post(&create)
        .form(&[
            ("resource_key", SEED_DROIT_RESOURCE_KEY),
            ("can_read", "true"),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "create: the form again");

    let edit = format!(
        "{base}{ADMIN_PREFIX}/users/{}/edit",
        crate::helpers::pk::pk(50)
    );
    let token = csrf(&client, &edit).await;
    let resp = client
        .post(&edit)
        .form(&[
            ("username", "editor"),
            ("email", "crawler@example.com"),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "edit: the form again");
}

/// An edit based on a stale copy of the row is refused instead of silently
/// overwriting someone else's change.
#[tokio::test]
#[serial]
async fn a_stale_edit_is_refused() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    db.execute_unprepared(
        "UPDATE eihwaz_users SET updated_at = '2026-10-01T10:00:00' WHERE username = 'editor'",
    )
    .await
    .unwrap();
    let client = login_superuser(&base).await;
    let edit = format!(
        "{base}{ADMIN_PREFIX}/users/{}/edit",
        crate::helpers::pk::pk(50)
    );
    let page = client
        .get(&edit)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let current = hidden_value(&page, "__original_updated_at").expect("timestamp in the form");

    let post = |stamp: String, name: &'static str| {
        let client = client.clone();
        let edit = edit.clone();
        async move {
            let token = csrf(&client, &edit).await;
            client
                .post(&edit)
                .form(&[
                    ("username", name),
                    ("email", "editor@example.com"),
                    ("__original_updated_at", stamp.as_str()),
                    ("csrf_token", token.as_str()),
                ])
                .send()
                .await
                .unwrap()
                .status()
        }
    };
    assert_eq!(
        post("2000-01-01T00:00:00".into(), "stale_name").await,
        200,
        "refused"
    );
    let name: String = db
        .query_one_raw(runique::sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!(
                "SELECT username FROM eihwaz_users WHERE id = {}",
                crate::helpers::pk::pk_sql_literal(50)
            ),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "username")
        .unwrap();
    assert_eq!(name, "editor", "nothing written");
    assert!(
        post(current, "fresh_name").await.is_redirection(),
        "an up-to-date edit goes through"
    );
}

/// After a save or a delete, the list comes back on the page the admin was on.
#[tokio::test]
#[serial]
async fn the_list_position_survives_edit_and_delete() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let client = login_superuser(&base).await;
    let edit = format!(
        "{base}{ADMIN_PREFIX}/users/{}/edit",
        crate::helpers::pk::pk(50)
    );

    let page = client
        .get(format!("{edit}?return_qs=page%3D2"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(hidden_value(&page, "return_qs").as_deref(), Some("page=2"));

    let location = |r: &reqwest::Response| r.headers()["location"].to_str().unwrap().to_string();
    for (qs, expected) in [
        ("page=2", "/admin/users/list?page=2"),
        ("", "/admin/users/list"),
    ] {
        let token = csrf(&client, &edit).await;
        let resp = client
            .post(&edit)
            .form(&[
                ("username", "editor"),
                ("email", "editor@example.com"),
                ("return_qs", qs),
                ("csrf_token", &token),
            ])
            .send()
            .await
            .unwrap();
        assert_eq!(location(&resp), expected, "edit, return_qs={qs:?}");
    }
    let token = csrf(&client, &edit).await;
    let resp = client
        .post(format!(
            "{base}{ADMIN_PREFIX}/users/{}/delete",
            crate::helpers::pk::pk(50)
        ))
        .form(&[("return_qs", "page=3"), ("csrf_token", &token)])
        .send()
        .await
        .unwrap();
    assert_eq!(location(&resp), "/admin/users/list?page=3", "delete");
}

async fn editor_row(db: &DatabaseConnection) -> (String, bool, String) {
    let row = db
        .query_one_raw(runique::sea_orm::Statement::from_string(
            db.get_database_backend(),
            format!(
                "SELECT password, is_staff, username FROM eihwaz_users WHERE id = {}",
                crate::helpers::pk::pk_sql_literal(50)
            ),
        ))
        .await
        .unwrap()
        .unwrap();
    (
        row.try_get("", "password").unwrap(),
        row.try_get("", "is_staff").unwrap(),
        row.try_get("", "username").unwrap(),
    )
}

async fn bulk(client: &reqwest::Client, base: &str, fields: &[(&str, &str)]) -> reqwest::Response {
    let list = format!("{base}{ADMIN_PREFIX}/users/list");
    let token = csrf(client, &list).await;
    let id = crate::helpers::pk::pk(50).to_string();
    let mut form = vec![("ids", id.as_str()), ("csrf_token", token.as_str())];
    form.extend_from_slice(fields);
    client
        .post(format!("{base}{ADMIN_PREFIX}/users/bulk"))
        .form(&form)
        .send()
        .await
        .unwrap()
}

/// A bulk write only sets what the bulk form or a configured group action
/// offers, each value checked by its field — written 2026-10-03: both bulk
/// paths used to hand every key of the request to the write function.
#[tokio::test]
#[serial]
async fn a_bulk_write_only_sets_what_the_admin_offers() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let client = login_superuser(&base).await;
    let before = editor_row(&db).await;

    // A hidden field of the form (the create form's injected password).
    bulk(
        &client,
        &base,
        &[
            ("bulk_action", "update-submit"),
            ("password", "chosen-by-attacker"),
        ],
    )
    .await;
    // A column the bulk form doesn't show at all.
    bulk(
        &client,
        &base,
        &[
            ("bulk_action", "update-submit"),
            ("is_superuser", "true"),
            ("is_staff", "false"),
        ],
    )
    .await;
    // A value its field refuses (a group that doesn't exist).
    bulk(
        &client,
        &base,
        &[("bulk_action", "update-submit"), ("groupes", "999")],
    )
    .await;
    // A group action nobody configured for `users`.
    bulk(
        &client,
        &base,
        &[("bulk_action", "group_set"), ("ga_is_staff", "false")],
    )
    .await;
    assert_eq!(editor_row(&db).await, before, "nothing written");
    assert_eq!(memberships(&db).await, [1], "memberships untouched");

    let resp = bulk(
        &client,
        &base,
        &[("bulk_action", "update-submit"), ("is_staff", "false")],
    )
    .await;
    assert!(resp.status().is_redirection());
    assert!(!editor_row(&db).await.1, "an offered field is applied");
}

async fn spawn_with_extra_routes(
    routes: Vec<(
        &'static str,
        &'static str,
        runique::admin::resource::CrudOperation,
        axum::routing::MethodRouter,
    )>,
) -> (String, DatabaseConnection) {
    let (router, dbc) = admin_server::build_admin_app_with_extra_routes(routes).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    (format!("http://{addr}"), dbc)
}

fn report_routes() -> Vec<(
    &'static str,
    &'static str,
    runique::admin::resource::CrudOperation,
    axum::routing::MethodRouter,
)> {
    use axum::routing::{get, post};
    use runique::admin::resource::CrudOperation;
    vec![
        (
            "/groupes/rapport",
            "groupes",
            CrudOperation::View,
            get(|| async { "rapport" }),
        ),
        (
            "/groupes/purge",
            "groupes",
            CrudOperation::Delete,
            post(|| async { "purgé" }),
        ),
        (
            "/users/export",
            "users",
            CrudOperation::List,
            get(|| async { "export" }),
        ),
    ]
}

/// A custom admin route is behind the right it declares, on the resource it
/// declares, like the generated pages — written 2026-10-03: custom routes only
/// checked that the visitor was signed-in staff.
#[tokio::test]
#[serial]
async fn custom_admin_routes_need_their_declared_right() {
    let (base, db) = spawn_with_extra_routes(report_routes()).await;
    add_editor(&db).await;
    let editor = login_editor(&base).await;

    let rapport = editor
        .get(format!("{base}{ADMIN_PREFIX}/groupes/rapport"))
        .send()
        .await
        .unwrap();
    assert_eq!(rapport.status(), 200, "read on groupes: granted");
    assert_eq!(rapport.text().await.unwrap(), "rapport");

    let export = editor
        .get(format!("{base}{ADMIN_PREFIX}/users/export"))
        .send()
        .await
        .unwrap();
    assert!(
        export.status().is_redirection(),
        "nothing on users: {}",
        export.status()
    );

    let token = csrf(&editor, &format!("{base}{ADMIN_PREFIX}/groupes/list")).await;
    let purge = editor
        .post(format!("{base}{ADMIN_PREFIX}/groupes/purge"))
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .unwrap();
    assert!(
        purge.status().is_redirection(),
        "read-only can't delete: {}",
        purge.status()
    );

    let root = login_superuser(&base).await;
    let export = root
        .get(format!("{base}{ADMIN_PREFIX}/users/export"))
        .send()
        .await
        .unwrap();
    assert_eq!(export.status(), 200, "superuser");
}

#[tokio::test]
#[serial]
#[should_panic(expected = "isn't registered")]
async fn a_custom_route_on_an_unknown_resource_is_a_build_error() {
    use runique::admin::resource::CrudOperation;
    let _ = admin_server::build_admin_app_with_extra_routes(vec![(
        "/rapport",
        "ghost",
        CrudOperation::View,
        axum::routing::get(|| async { "x" }),
    )])
    .await;
}

async fn sql(db: &DatabaseConnection, statement: &str) {
    db.execute_unprepared(statement).await.unwrap();
}

/// The session says who signed in, never what they may still do: account
/// state and rights are read from the database on every admin request, so a
/// change made anywhere — here, straight in SQL — applies to the next one.
#[tokio::test]
#[serial]
async fn a_revoked_right_is_refused_on_the_next_request() {
    let (base, db) = spawn().await;
    add_editor(&db).await;
    let editor = login_editor(&base).await;
    let list = format!("{base}{ADMIN_PREFIX}/groupes/list");
    assert_eq!(editor.get(&list).send().await.unwrap().status(), 200);

    sql(&db, "UPDATE eihwaz_groupes_droits SET can_read = 0").await;
    let resp = editor.get(&list).send().await.unwrap();
    assert!(resp.status().is_redirection(), "{}", resp.status());
}

#[tokio::test]
#[serial]
async fn a_deactivated_or_demoted_account_loses_its_session() {
    for revoke in [
        "UPDATE eihwaz_users SET is_active = 0 WHERE username = 'editor'",
        "UPDATE eihwaz_users SET is_staff = 0 WHERE username = 'editor'",
    ] {
        let (base, db) = spawn().await;
        add_editor(&db).await;
        let editor = login_editor(&base).await;
        let list = format!("{base}{ADMIN_PREFIX}/groupes/list");
        assert_eq!(editor.get(&list).send().await.unwrap().status(), 200);

        sql(&db, revoke).await;
        let resp = editor.get(&list).send().await.unwrap();
        assert_eq!(
            resp.headers()["location"].to_str().unwrap(),
            format!("{ADMIN_PREFIX}/login"),
            "{revoke}"
        );

        // Closed, not just refused: restoring the account doesn't bring the
        // old session back.
        sql(
            &db,
            "UPDATE eihwaz_users SET is_active = 1, is_staff = 1 WHERE username = 'editor'",
        )
        .await;
        let resp = editor.get(&list).send().await.unwrap();
        assert!(
            resp.status().is_redirection(),
            "{revoke}: session still open"
        );
    }
}

/// The first activation is the owner's: an admin ticking "active" on an
/// account never activated is refused with an explanation, the account stays
/// pending. Reactivating a blocked account remains the admin's to do.
#[tokio::test]
#[serial]
async fn an_admin_cannot_activate_an_account_its_owner_never_activated() {
    let (base, db) = spawn().await;
    db.execute_unprepared(&format!(
        "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
         VALUES ({}, 'invited', 'invited@example.com', 'h', 0, 0, 0, NULL), \
                ({}, 'blocked', 'blocked@example.com', 'h', 0, 0, 0, '2026-01-01 00:00:00')",
        crate::helpers::pk::pk_sql_literal(60),
        crate::helpers::pk::pk_sql_literal(61)
    ))
    .await
    .unwrap();
    let client = login_superuser(&base).await;

    let activate = |n: u32, name: &'static str| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let url = format!(
                "{base}{ADMIN_PREFIX}/users/{}/edit",
                crate::helpers::pk::pk(n)
            );
            let token = csrf(&client, &url).await;
            client
                .post(&url)
                .form(&[
                    ("username", name),
                    ("email", &format!("{name}@example.com")),
                    ("is_active", "true"),
                    ("csrf_token", token.as_str()),
                ])
                .send()
                .await
                .unwrap()
        }
    };
    let is_active = |name: &'static str| {
        let db = db.clone();
        async move {
            db.query_one_raw(runique::sea_orm::Statement::from_string(
                db.get_database_backend(),
                format!("SELECT is_active FROM eihwaz_users WHERE username = '{name}'"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<bool>("", "is_active")
            .unwrap()
        }
    };

    let refused = activate(60, "invited").await;
    assert_eq!(refused.status(), 200, "the form again");
    let page = refused.text().await.unwrap();
    assert!(
        page.contains(&*runique::utils::trad::t("admin.user.not_activated")),
        "explained"
    );
    assert!(!is_active("invited").await, "still pending");

    assert!(activate(61, "blocked").await.status().is_redirection());
    assert!(
        is_active("blocked").await,
        "a blocked account is unblocked by the admin"
    );
}
