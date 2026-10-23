//! The audit trail (`eihwaz_history`): one row per traced write, written by
//! `save_as` in the write's own transaction — the admin's writes and the
//! developer's `save_as` alike.
use crate::utils::{
    aliases::StrMap,
    constante::session_key::session::{CSRF_TOKEN_KEY, FORM_NAME_KEY},
    pk::Pk,
};
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde_json::Value;

// ─── SeaORM Entity — eihwaz_history ─────────────────────────────────────────

/// SeaORM entity for the `eihwaz_history` audit log table: one row per traced
/// write, recording who performed it, on which resource/object, and an
/// optional JSON diff of the changed fields.
#[derive(Clone, Debug, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "eihwaz_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub resource_key: String,
    pub object_pk: String,
    /// The row's label when the action happened, kept: the trace stays
    /// readable once the row is renamed or deleted.
    pub object_repr: String,
    pub action: String,
    pub user_id: Pk,
    pub username: String,
    pub created_at: chrono::NaiveDateTime,
    pub summary: Option<String>,
    pub batch_id: Option<String>,
}

/// No relations are defined for the `eihwaz_history` entity.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Input to [`log_action`]: the actor, the resource/object acted on,
/// the action name, and an optional diff summary to persist in the audit log.
pub struct ActionLog<'a> {
    pub user_id: Pk,
    pub username: &'a str,
    pub resource_key: &'a str,
    pub object_pk: &'a str,
    pub object_repr: &'a str,
    pub action: &'a str,
    pub summary: Option<String>,
    pub batch_id: Option<String>,
}

/// Longest label kept in `object_repr`, in characters.
pub const OBJECT_REPR_MAX: usize = 255;

/// Inserts one row in `eihwaz_history`, on `db` — the write's transaction, so
/// the row and its trace are committed or rolled back together: no write
/// without its trace.
pub async fn log_action<C: ConnectionTrait>(db: &C, log: ActionLog<'_>) -> Result<(), DbErr> {
    let entry = ActiveModel {
        resource_key: Set(log.resource_key.to_string()),
        object_pk: Set(log.object_pk.to_string()),
        // MySQL's VARCHAR(255) refuses a longer value: a long label is cut,
        // never the reason a write fails.
        object_repr: Set(log.object_repr.chars().take(OBJECT_REPR_MAX).collect()),
        action: Set(log.action.to_string()),
        user_id: Set(log.user_id),
        username: Set(log.username.to_string()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        summary: Set(log.summary),
        batch_id: Set(log.batch_id),
        ..Default::default()
    };
    Entity::insert(entry).exec(db).await.map(|_| ())
}

/// Marker substituted for a sensitive field's value in an audit diff.
/// The field itself stays listed — knowing *that* a password changed is
/// legitimate audit information; knowing its value, never.
pub const REDACTED: &str = "••••••";

/// Name fragments that make a field sensitive.
///
/// The history table is readable by anyone with the history permission, even
/// without access to the table in question: copying a password hash into it
/// takes it out of the one table meant to hold it and hands out offline attack
/// material. The check is name-based because that's the only signal available
/// on **every** write path — including a custom `update_fn` that never goes
/// through any `PasswordField`.
const SENSITIVE_HINTS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "private_key",
];

/// `true` if this field's value must never enter the audit trail.
pub fn is_sensitive_key(key: &str) -> bool {
    let k = key.to_lowercase();
    SENSITIVE_HINTS.iter().any(|hint| k.contains(hint))
}

/// Redacts sensitive values in a `{field: {old, new}}` changes map.
///
/// **Single** pass-through point: every history write path (plain edit, bulk
/// with or without `get_fn`) goes through here, so a future write path can't
/// bypass the rule by forgetting a local filter.
pub fn redact_sensitive(changes: &mut serde_json::Map<String, Value>) {
    for (key, entry) in changes.iter_mut() {
        if !is_sensitive_key(key) {
            continue;
        }
        if let Value::Object(fields) = entry {
            for v in fields.values_mut() {
                *v = Value::String(REDACTED.to_string());
            }
        } else {
            *entry = Value::String(REDACTED.to_string());
        }
    }
}

/// Compares an old DB object (`get_fn` result) against submitted form fields.
/// Returns a compact JSON string of changed fields: `{"title":{"old":"a","new":"b"}}`.
/// Sensitive fields are listed, but their values are replaced with [`REDACTED`].
/// Returns `None` if nothing changed or old state is unavailable.
pub fn diff_fields(old: &Value, body: &StrMap) -> Option<String> {
    let Value::Object(map) = old else {
        return None;
    };
    let mut changes = std::collections::BTreeMap::new();
    for (k, new_val) in body {
        if k == CSRF_TOKEN_KEY || k == FORM_NAME_KEY || k == "__original_updated_at" {
            continue;
        }
        let old_str = match map.get(k) {
            Some(Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => continue,
        };
        if old_str != *new_val {
            changes.insert(
                k.clone(),
                serde_json::json!({"old": old_str, "new": new_val}),
            );
        }
    }
    if changes.is_empty() {
        return None;
    }
    let mut changes: serde_json::Map<String, Value> = changes.into_iter().collect();
    redact_sensitive(&mut changes);
    serde_json::to_string(&changes).ok()
}

#[cfg(test)]
mod tests {
    use super::diff_fields;
    use crate::utils::{
        aliases::StrMap,
        constante::session_key::session::{CSRF_TOKEN_KEY, FORM_NAME_KEY},
    };
    use serde_json::json;

    fn body(pairs: &[(&str, &str)]) -> StrMap {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn the_form_name_and_csrf_never_count_as_changes() {
        let old = json!({ FORM_NAME_KEY: "x", CSRF_TOKEN_KEY: "x", "title": "a" });
        let new = body(&[
            (FORM_NAME_KEY, "edit"),
            (CSRF_TOKEN_KEY, "tok"),
            ("title", "a"),
        ]);
        assert_eq!(diff_fields(&old, &new), None);
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn a_long_label_is_cut_never_refused() {
        use super::{ActionLog, Entity, OBJECT_REPR_MAX, log_action};
        use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder};

        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        db.execute_unprepared(
            &crate::auth::migrations_table::create_eihwaz_history_table()
                .to_string(sea_orm::sea_query::SqliteQueryBuilder),
        )
        .await
        .unwrap();
        for len in [OBJECT_REPR_MAX, OBJECT_REPR_MAX + 1] {
            let label = "é".repeat(len);
            log_action(
                &db,
                ActionLog {
                    user_id: crate::utils::pk::Pk::default(),
                    username: "u",
                    resource_key: "articles",
                    object_pk: "1",
                    object_repr: &label,
                    action: "edit",
                    summary: None,
                    batch_id: None,
                },
            )
            .await
            .expect("a long label never fails the write");
        }
        let rows = Entity::find()
            .order_by_asc(super::Column::Id)
            .all(&db)
            .await
            .unwrap();
        let kept: Vec<usize> = rows.iter().map(|r| r.object_repr.chars().count()).collect();
        assert_eq!(kept, [OBJECT_REPR_MAX, OBJECT_REPR_MAX]);
    }

    #[test]
    fn a_real_column_change_is_still_reported() {
        let old = json!({ "title": "a" });
        let new = body(&[(FORM_NAME_KEY, "edit"), ("title", "b")]);
        let diff = diff_fields(&old, &new).expect("title changed");
        assert!(diff.contains("title") && !diff.contains(FORM_NAME_KEY));
    }
}
