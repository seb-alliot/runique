//! Parent-detail inline sub-lists: every resource declared as a scoped child of
//! the viewed parent (via `ParentScope`), listed filtered to the parent id.
use std::collections::HashMap;

use serde::Serialize;

use super::PrototypeAdminState;
use super::ResourcePerms;
use super::format_datetime;
use super::handle_list::resolve_columns;
use crate::admin::helper::resource_entry::{ListParams, ResourceEntry, SortDir};
use crate::auth::session::CurrentUser;
use crate::utils::aliases::ADb;

/// A scoped child rendered as a sub-list under its parent's detail page.
#[derive(Serialize)]
pub(super) struct InlineList {
    /// Child resource key (e.g. `"droits"`).
    pub key: String,
    /// Human title for the sub-list heading.
    pub title: String,
    /// Scope-aware base path for the child's action URLs
    /// (`{prefix}/{parent_key}/{parent_id}/{child_key}`).
    pub base: String,
    /// Per-action rights of the current user on the child resource — the inline
    /// buttons are gated on these so the UI matches the enforced policy.
    pub can_create: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub columns: Vec<String>,
    pub column_labels: HashMap<String, String>,
    /// Scoped rows, with the composite parent prefix stripped from `id`.
    pub rows: Vec<serde_json::Value>,
}

/// Builds the inline sub-lists for `parent_entry`'s detail page: scans the
/// registry for resources scoped to it, lists each filtered to `parent_id`, and
/// keeps only the children the user is allowed to read.
pub(super) async fn build_inlines(
    db: ADb,
    state: &PrototypeAdminState,
    parent_entry: &ResourceEntry,
    parent_id: &str,
    current_user: &CurrentUser,
) -> Vec<InlineList> {
    let prefix = state.config.prefix.trim_end_matches('/');
    let mut inlines = Vec::new();

    for child in state.registry.all() {
        let Some(scope) = &child.meta.parent_scope else {
            continue;
        };
        if scope.parent_key != parent_entry.meta.key {
            continue;
        }
        if !current_user.is_superuser && !current_user.can_access_resource(child.meta.key) {
            continue;
        }
        let Some(list_fn) = &child.list_fn else {
            continue;
        };

        let params = ListParams {
            offset: 0,
            limit: 500,
            sort_by: None,
            sort_dir: SortDir::Asc,
            search: None,
            column_filters: Vec::new(),
            scope: Some((scope.fk_col.to_string(), parent_id.to_string())),
        };
        let mut rows = match list_fn(db.clone(), params).await {
            Ok(rows) => rows,
            Err(e) => {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .admin
                    .as_ref()
                    .and_then(|a| a.list)
                {
                    crate::runique_log!(
                        level,
                        resource = child.meta.key,
                        error = %e,
                        "inline list_fn failed — sub-list omitted"
                    );
                }
                continue;
            }
        };

        crate::admin::helper::resolve_fk_labels(db.as_ref(), &mut rows, &child.meta.fk_display)
            .await;
        if let Some(apply_enum_labels) = child.enum_label_fn {
            for row in &mut rows {
                apply_enum_labels(row);
            }
        }
        // Composite child: expose the local id so row URLs are `{base}/{local}/…`.
        if scope.local_key.is_some() {
            for row in &mut rows {
                if let Some(id_str) = row.get("id").and_then(|v| v.as_str()) {
                    let local = id_str
                        .split_once(':')
                        .map_or(id_str, |(_, local)| local)
                        .to_string();
                    if let serde_json::Value::Object(map) = row {
                        map.insert("id".to_string(), serde_json::Value::String(local));
                    }
                }
            }
        }
        for row in &mut rows {
            format_datetime(row);
        }

        // The FK column is constant across a scoped sub-list (all == parent_id),
        // so drop it here — only in the inline, never in the flat top-level view.
        let (mut columns, column_labels) = resolve_columns(child, &rows);
        columns.retain(|c| c != scope.fk_col);
        let perms = ResourcePerms::resolve(current_user, child.meta.key);
        inlines.push(InlineList {
            key: child.meta.key.to_string(),
            title: child.meta.title.to_string(),
            base: format!(
                "{}/{}/{}/{}",
                prefix, parent_entry.meta.key, parent_id, child.meta.key
            ),
            can_create: perms.can_create,
            can_update: perms.can_update,
            can_delete: perms.can_delete,
            columns,
            column_labels,
            rows,
        });
    }

    inlines
}
