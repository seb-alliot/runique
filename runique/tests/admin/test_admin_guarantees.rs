//! Admin guarantees, written from cargo-mutants survivors (2026-10-02): the
//! audit trail never stores a secret, every admin action is recorded with what
//! it did, and role checks grant only what they list.
use crate::helpers::{admin_server, db, pk::pk};
use runique::admin::history::{
    AdminActionLog, diff_fields, is_sensitive_key, log_admin_action, redact_sensitive,
};
use runique::db::ADb;
use runique::utils::aliases::StrMap;

// ── History: secrets never enter the audit trail ────────────────────────────

#[test]
fn secret_looking_keys_are_recognised() {
    for key in [
        "password",
        "user_password",
        "API_KEY",
        "reset_token",
        "private_key",
    ] {
        assert!(is_sensitive_key(key), "{key}");
    }
    for key in ["title", "email", "username"] {
        assert!(!is_sensitive_key(key), "{key}");
    }
}

#[test]
fn redaction_hides_secret_values_and_keeps_the_rest() {
    let mut changes = serde_json::json!({
        "password": { "old": "$argon2id$old", "new": "$argon2id$new" },
        "api_key": "raw-key",
        "title": { "old": "a", "new": "b" }
    });
    redact_sensitive(changes.as_object_mut().unwrap());
    let shown = changes.to_string();
    assert!(
        !shown.contains("argon2") && !shown.contains("raw-key"),
        "{shown}"
    );
    assert_eq!(changes["title"]["new"], "b");
}

#[test]
fn diff_never_records_form_bookkeeping_keys() {
    let old = serde_json::json!({ "csrf_token": "old", "__original_updated_at": "t0" });
    let body: StrMap = [("csrf_token", "new"), ("__original_updated_at", "t1")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(diff_fields(&old, &body), None);
}

#[test]
fn diff_lists_only_changed_fields_with_secrets_redacted() {
    let old = serde_json::json!({ "title": "a", "count": 2, "password": "$argon2id$old" });
    let body: StrMap = [
        ("title", "b"),
        ("count", "2"),
        ("password", "$argon2id$new"),
        ("csrf_token", "tok"),
        ("unknown", "x"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let diff: serde_json::Value = serde_json::from_str(&diff_fields(&old, &body).unwrap()).unwrap();
    assert_eq!(diff["title"], serde_json::json!({ "old": "a", "new": "b" }));
    assert!(
        diff.get("count").is_none(),
        "unchanged (number vs string compared as text)"
    );
    assert!(diff.get("csrf_token").is_none() && diff.get("unknown").is_none());
    assert!(!diff.to_string().contains("argon2"), "{diff}");

    let same: StrMap = [("title".to_string(), "a".to_string())]
        .into_iter()
        .collect();
    assert_eq!(diff_fields(&old, &same), None, "nothing changed");
    assert_eq!(
        diff_fields(&serde_json::json!("not an object"), &same),
        None
    );
}

#[tokio::test]
async fn an_admin_action_is_recorded_with_everything_it_did() {
    let conn = db::fresh_db().await;
    db::exec(&conn, admin_server::HISTORY_DDL).await;
    let db = ADb::from_connection(conn.clone());
    log_admin_action(
        &db,
        AdminActionLog {
            user_id: pk(3),
            username: "editor",
            resource_key: "articles",
            object_pk: "42",
            action: "update",
            summary: Some(r#"{"title":{"old":"a","new":"b"}}"#.into()),
            batch_id: Some("batch-1".into()),
        },
    )
    .await;
    use runique::sea_orm::{ConnectionTrait, Statement};
    let row = conn
        .query_one_raw(Statement::from_string(
            conn.get_database_backend(),
            "SELECT resource_key, object_pk, action, username, summary, batch_id, created_at FROM eihwaz_history",
        ))
        .await
        .unwrap()
        .expect("one row");
    let get = |c: &str| row.try_get::<Option<String>>("", c).unwrap();
    assert_eq!(get("resource_key").as_deref(), Some("articles"));
    assert_eq!(get("object_pk").as_deref(), Some("42"));
    assert_eq!(get("action").as_deref(), Some("update"));
    assert_eq!(get("username").as_deref(), Some("editor"));
    assert!(get("summary").unwrap().contains("title"));
    assert_eq!(get("batch_id").as_deref(), Some("batch-1"));
    assert!(get("created_at").is_some());
}

// ── Role-based permissions ─────────────────────────────────────────────────

/// A right left behind by a resource that's no longer registered is dropped at
/// boot — otherwise it would come back as a live grant if a resource ever got
/// that key again.
#[tokio::test]
#[serial_test::serial]
async fn boot_prunes_rights_on_unregistered_resources() {
    use crate::helpers::admin_server::{ORPHAN_DROIT_RESOURCE_KEY, SEED_DROIT_RESOURCE_KEY};
    let (_router, db) = crate::helpers::admin_server::build_admin_app().await;
    let count = |key: &'static str| {
        let db = db.clone();
        async move {
            use runique::sea_orm::{ConnectionTrait, Statement};
            db.query_one_raw(Statement::from_string(
                db.get_database_backend(),
                format!(
                    "SELECT COUNT(*) AS n FROM eihwaz_groupes_droits WHERE resource_key = '{key}'"
                ),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "n")
            .unwrap()
        }
    };
    assert_eq!(count(ORPHAN_DROIT_RESOURCE_KEY).await, 0);
    assert_eq!(
        count(SEED_DROIT_RESOURCE_KEY).await,
        1,
        "a registered resource keeps its rights"
    );
}
