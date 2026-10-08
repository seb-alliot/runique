//! The built-in admin resources (`users`, `groupes`, `droits`): what their data
//! functions write and read, called directly on the test schema. Written from
//! cargo-mutants survivors (2026-10-08): admin/builtin/user.rs, groupe.rs, droit.rs.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, GROUPES_DDL, GROUPES_DROITS_DDL, USERS_DDL, USERS_GROUPES_DDL, build_admin_app,
    login_as_superuser,
};
use crate::helpers::db;
use crate::helpers::pk::{pk, pk_sql_literal};
use runique::admin::builtin_resources;
use runique::admin::helper::resource_entry::{ListParams, ResourceEntry, SortDir};
use runique::db::ADb;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use runique::utils::aliases::StrMap;
use serial_test::serial;

async fn schema() -> (DatabaseConnection, ADb) {
    let conn = db::fresh_db().await;
    for ddl in [
        USERS_DDL,
        GROUPES_DDL,
        GROUPES_DROITS_DDL,
        USERS_GROUPES_DDL,
    ] {
        db::exec(&conn, ddl).await;
    }
    let adb = ADb::from_connection(conn.clone());
    (conn, adb)
}

fn entry(key: &str) -> ResourceEntry {
    builtin_resources()
        .into_iter()
        .find(|e| e.meta.key == key)
        .expect("built-in resource")
}

fn data(pairs: &[(&str, &str)]) -> StrMap {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn params(sort_by: &str, sort_dir: SortDir, scope: Option<(&str, &str)>) -> ListParams {
    ListParams {
        offset: 0,
        limit: 50,
        sort_by: Some(sort_by.to_string()),
        sort_dir,
        search: None,
        column_filters: Vec::new(),
        scope: scope.map(|(c, v)| (c.to_string(), v.to_string())),
    }
}

/// One row of a raw query, as text per column.
async fn row(conn: &DatabaseConnection, sql: &str) -> Vec<Option<String>> {
    let r = conn
        .query_one_raw(Statement::from_string(conn.get_database_backend(), sql))
        .await
        .unwrap()
        .expect("a row");
    (0..r.column_names().len())
        .map(|i| {
            r.try_get_by_index::<Option<String>>(i)
                .ok()
                .flatten()
                .or_else(|| {
                    r.try_get_by_index::<Option<i64>>(i)
                        .ok()
                        .flatten()
                        .map(|n| n.to_string())
                })
        })
        .collect()
}

fn column(rows: &[serde_json::Value], col: &str) -> Vec<String> {
    rows.iter()
        .map(|r| match &r[col] {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect()
}

// ── users ────────────────────────────────────────────────────────────────────

/// Created from the admin: staff as asked, never superuser whatever the form
/// says, timestamps set.
#[tokio::test]
async fn admin_created_user_is_staff_as_asked_never_superuser() {
    let (conn, adb) = schema().await;
    let create = entry("users").create_fn.unwrap();
    create(
        adb.clone(),
        data(&[
            ("username", "alice"),
            ("email", "alice@example.com"),
            ("password", "$argon2id$hash"),
            ("is_staff", "on"),
            ("is_superuser", "on"),
        ]),
    )
    .await
    .unwrap();
    create(
        adb,
        data(&[
            ("username", "bob"),
            ("email", "bob@example.com"),
            ("password", "h"),
        ]),
    )
    .await
    .unwrap();

    let alice = row(
        &conn,
        "SELECT is_staff, is_superuser, created_at, updated_at FROM eihwaz_users WHERE username = 'alice'",
    )
    .await;
    assert_eq!(alice[0].as_deref(), Some("1"), "staff as asked");
    assert_eq!(
        alice[1].as_deref(),
        Some("0"),
        "never superuser from the admin"
    );
    assert!(
        alice[2].is_some() && alice[3].is_some(),
        "timestamps set: {alice:?}"
    );
    let bob = row(
        &conn,
        "SELECT is_staff FROM eihwaz_users WHERE username = 'bob'",
    )
    .await;
    assert_eq!(bob[0].as_deref(), Some("0"), "not staff when not asked");
}

/// The edit form's update writes every field it owns, on the right row.
#[tokio::test]
async fn admin_edit_writes_username_email_active_and_staff() {
    let (conn, adb) = schema().await;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, 'bob', 'bob@example.com', 'h', 0, 0, 0, '2026-01-01 00:00:00'), \
                    ({}, 'carol', 'carol@example.com', 'h', 0, 0, 0, NULL)",
            pk_sql_literal(5),
            pk_sql_literal(6)
        ),
    )
    .await;
    let update = entry("users").update_fn.unwrap();
    update(
        adb,
        pk(5).to_string(),
        data(&[
            ("username", "bobby"),
            ("email", "bobby@example.com"),
            ("is_active", "on"),
            ("is_staff", "on"),
        ]),
    )
    .await
    .unwrap();

    let bob = row(
        &conn,
        "SELECT username, email, is_active, is_staff, is_superuser, updated_at FROM eihwaz_users WHERE username = 'bobby'",
    )
    .await;
    assert_eq!(
        &bob[..5],
        &[
            Some("bobby".into()),
            Some("bobby@example.com".into()),
            Some("1".into()),
            Some("1".into()),
            Some("0".into()),
        ]
    );
    assert!(bob[5].is_some(), "updated_at set");
    let carol = row(
        &conn,
        "SELECT username, is_staff FROM eihwaz_users WHERE username = 'carol'",
    )
    .await;
    assert_eq!(
        carol,
        [Some("carol".into()), Some("0".into())],
        "other rows untouched"
    );
}

/// A partial update (bulk, inline toggle) stamps `updated_at` too.
#[tokio::test]
async fn admin_partial_update_stamps_updated_at() {
    let (conn, adb) = schema().await;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser) \
             VALUES ({}, 'bob', 'bob@example.com', 'h', 0, 0, 0)",
            pk_sql_literal(5)
        ),
    )
    .await;
    let partial = entry("users").partial_update_fn.unwrap();
    partial(adb, pk(5).to_string(), data(&[("is_staff", "on")]))
        .await
        .unwrap();
    let bob = row(
        &conn,
        "SELECT is_staff, updated_at FROM eihwaz_users WHERE username = 'bob'",
    )
    .await;
    assert_eq!(bob[0].as_deref(), Some("1"));
    assert!(bob[1].is_some(), "updated_at set");
}

/// Each built-in list sorts both ways on a listed column.
#[tokio::test]
async fn builtin_lists_sort_both_ways() {
    let (conn, adb) = schema().await;
    db::exec(
        &conn,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser) \
             VALUES ({}, 'anna', 'a@x.fr', 'h', 0, 0, 0), ({}, 'zoe', 'z@x.fr', 'h', 0, 0, 0)",
            pk_sql_literal(1),
            pk_sql_literal(2)
        ),
    )
    .await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (1, 'Auteurs'), (2, 'Zélés')",
    )
    .await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) \
         VALUES (1, 'articles', 0, 1, 0, 0, 0, 0), (1, 'pages', 0, 1, 0, 0, 0, 0)",
    )
    .await;

    for (key, col) in [
        ("users", "username"),
        ("groupes", "nom"),
        ("droits", "resource_key"),
    ] {
        let list = entry(key).list_fn.unwrap();
        let asc = column(
            &list(adb.clone(), params(col, SortDir::Asc, None))
                .await
                .unwrap(),
            col,
        );
        let desc = column(
            &list(adb.clone(), params(col, SortDir::Desc, None))
                .await
                .unwrap(),
            col,
        );
        assert_eq!(asc.len(), 2, "{key}: {asc:?}");
        let mut reversed = asc.clone();
        reversed.reverse();
        assert_eq!(desc, reversed, "{key}");
        assert!(asc[0] < asc[1], "{key}: {asc:?}");
    }
}

// ── droits ───────────────────────────────────────────────────────────────────

/// A right is listed under its composite id `group:resource`, and a list
/// nested under a group shows (and counts) only that group's rights.
#[tokio::test]
async fn droits_list_ids_and_group_scope() {
    let (conn, adb) = schema().await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (1, 'A'), (2, 'B')",
    )
    .await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) \
         VALUES (1, 'articles', 0, 1, 0, 0, 0, 0), (2, 'pages', 0, 1, 0, 0, 0, 0)",
    )
    .await;
    let droits = entry("droits");
    let list = droits.list_fn.unwrap();
    let count = droits.count_fn.unwrap();

    let all = list(adb.clone(), params("resource_key", SortDir::Asc, None))
        .await
        .unwrap();
    assert_eq!(column(&all, "id"), ["1:articles", "2:pages"]);

    let scoped = list(
        adb.clone(),
        params("resource_key", SortDir::Asc, Some(("groupe_id", "2"))),
    )
    .await
    .unwrap();
    assert_eq!(
        column(&scoped, "id"),
        ["2:pages"],
        "only the parent group's rights"
    );
    let n = count(
        adb,
        None,
        Vec::new(),
        Some(("groupe_id".into(), "2".into())),
    )
    .await
    .unwrap();
    assert_eq!(n, 1, "the count is scoped the same way");
}

/// The resource checkboxes send `",posts"` when the first box is empty: the
/// first non-empty key is the one kept.
#[tokio::test]
async fn droit_update_keeps_the_first_non_empty_resource_key() {
    let (conn, adb) = schema().await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (1, 'A')",
    )
    .await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) \
         VALUES (1, 'articles', 0, 1, 0, 0, 0, 0)",
    )
    .await;
    let update = entry("droits").update_fn.unwrap();
    update(
        adb,
        "1:articles".to_string(),
        data(&[("resource_key", " ,posts")]),
    )
    .await
    .unwrap();
    let r = row(
        &conn,
        "SELECT resource_key FROM eihwaz_groupes_droits WHERE groupe_id = 1",
    )
    .await;
    assert_eq!(r[0].as_deref(), Some("posts"));
}

/// The right's edit page preselects its own group, and only that one.
#[tokio::test]
#[serial]
async fn droit_edit_page_preselects_only_its_group() {
    let (router, conn) = build_admin_app().await;
    db::exec(
        &conn,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'Lecteurs')",
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;

    let html = client
        .get(format!("{base}{ADMIN_PREFIX}/droits/1:groupes/edit"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(html.contains(r#"<option value="1" selected"#), "{html}");
    assert!(
        !html.contains(r#"<option value="2" selected"#),
        "another group is not preselected"
    );
}
