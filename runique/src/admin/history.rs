use crate::utils::{
    aliases::{ADb, StrMap},
    constante::session_key::session::CSRF_TOKEN_KEY,
    pk::Pk,
};
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde_json::Value;

// ─── SeaORM Entity — eihwaz_history ─────────────────────────────────────────

/// SeaORM entity for the `eihwaz_history` audit log table: one row per admin
/// CRUD action, recording who performed it, on which resource/object, and an
/// optional JSON diff of the changed fields.
#[derive(Clone, Debug, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "eihwaz_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub resource_key: String,
    pub object_pk: String,
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

/// Input to [`log_admin_action`]: the actor, the resource/object acted on,
/// the action name, and an optional diff summary to persist in the audit log.
pub struct AdminActionLog<'a> {
    pub user_id: Pk,
    pub username: &'a str,
    pub resource_key: &'a str,
    pub object_pk: &'a str,
    pub action: &'a str,
    pub summary: Option<String>,
    pub batch_id: Option<String>,
}

/// Fire-and-forget: inserts one row in `eihwaz_history`.
/// A failed audit insert must never break the request — but it is **logged**
/// (audit row lost), never silently dropped.
pub async fn log_admin_action(db: &ADb, log: AdminActionLog<'_>) {
    let now = chrono::Utc::now().naive_utc();
    let resource_key = log.resource_key.to_string();
    let object_pk = log.object_pk.to_string();
    let entry = ActiveModel {
        resource_key: Set(resource_key.clone()),
        object_pk: Set(object_pk.clone()),
        action: Set(log.action.to_string()),
        user_id: Set(log.user_id),
        username: Set(log.username.to_string()),
        created_at: Set(now),
        summary: Set(log.summary),
        batch_id: Set(log.batch_id),
        ..Default::default()
    };
    if let Err(e) = Entity::insert(entry).exec(db).await {
        tracing::warn!(
            resource = %resource_key,
            object_pk = %object_pk,
            error = %e,
            "history insert failed (audit row lost)"
        );
    }
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
        if k == CSRF_TOKEN_KEY || k == "__original_updated_at" {
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
