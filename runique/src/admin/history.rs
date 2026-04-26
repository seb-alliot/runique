use crate::utils::{
    aliases::{ADb, StrMap},
    constante::session_key::session::CSRF_TOKEN_KEY,
    pk::Pk,
};
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde_json::Value;

// ─── SeaORM Entity — eihwaz_history ─────────────────────────────────────────

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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Fire-and-forget: inserts one row in `eihwaz_history`.
/// Errors are silently swallowed — a log failure must never break the request.
pub async fn log_admin_action(
    db: &ADb,
    user_id: Pk,
    username: &str,
    resource_key: &str,
    object_pk: &str,
    action: &str,
    summary: Option<String>,
) {
    let now = chrono::Utc::now().naive_utc();
    let entry = ActiveModel {
        resource_key: Set(resource_key.to_string()),
        object_pk: Set(object_pk.to_string()),
        action: Set(action.to_string()),
        user_id: Set(user_id),
        username: Set(username.to_string()),
        created_at: Set(now),
        summary: Set(summary),
        ..Default::default()
    };
    let _ = Entity::insert(entry).exec(db.as_ref()).await;
}

/// Compares an old DB object (`get_fn` result) against submitted form fields.
/// Returns a compact JSON string of changed fields: `{"title":{"old":"a","new":"b"}}`.
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
        None
    } else {
        serde_json::to_string(&changes).ok()
    }
}
