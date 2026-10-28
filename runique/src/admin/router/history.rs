//! `GET {admin}/history` and `GET {admin}/history/{id}`: the writes the admin
//! traced, for the models the account fully reads, each diff cut to the
//! columns it reads.
use super::AdminPaths;
use super::context::{
    HistoryChange, HistoryContext, HistoryEntryContext, HistoryLinks, HistoryResource, HistoryRow,
    put,
};
use super::{chrome, failure};
use crate::admin::builder::registered::{AdminError, HistoryScope};
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::auth::current_user::CurrentUser;
use crate::auth::gate::Refusal;
use crate::auth::gate::refusal::RefusalReason;
use crate::auth::history;
use crate::context::template::Request;
use crate::utils::aliases::{AppResult, StrMap};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use std::sync::Arc;

/// Entries per page.
const PAGE_SIZE: u64 = 50;

/// `entry` as the pages show it, for the scope of its model.
fn row(entry: &history::Model, scope: &HistoryScope, paths: &AdminPaths) -> HistoryRow {
    let has_changes = !changes(entry, scope).is_empty();
    HistoryRow {
        id: entry.id,
        created_at: entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        username: entry.username.clone(),
        resource_key: entry.resource_key.clone(),
        resource_title: scope.title.clone(),
        object_pk: entry.object_pk.clone(),
        object_label: (scope.label_readable && entry.object_repr != entry.object_pk)
            .then(|| entry.object_repr.clone()),
        action: entry.action.clone(),
        has_changes,
        links: HistoryLinks {
            user: paths.history_with(&[("user", &entry.username)]),
            object: paths.history_with(&[
                ("resource", &entry.resource_key),
                ("object", &entry.object_pk),
            ]),
            entry: has_changes.then(|| paths.history_entry_of(entry.id)),
        },
    }
}

/// The columns `entry` changed that `scope` lets the account read.
fn changes(entry: &history::Model, scope: &HistoryScope) -> Vec<HistoryChange> {
    let shown = |value: Option<&serde_json::Value>| match value {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    };
    let Some(summary) = entry
        .summary
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(s).ok())
    else {
        return Vec::new();
    };
    summary
        .into_iter()
        .filter(|(column, _)| scope.readable.allows(column))
        .map(|(column, diff)| HistoryChange {
            old: shown(diff.get("old")),
            new: shown(diff.get("new")),
            column,
        })
        .collect()
}

pub(crate) async fn history_view(
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    mut req: Request,
) -> AppResult<Response> {
    let db = req.engine.db.clone();
    let scopes = site.history_scopes(&db, &user);
    let param = |key: &str| params.get(key).cloned().unwrap_or_default();
    let (resource, object, action, username) = (
        param("resource"),
        param("object"),
        param("action"),
        param("user"),
    );
    let page = params
        .get("page")
        .and_then(|p| p.parse::<u64>().ok())
        .unwrap_or(1)
        .max(1);

    // Only the models this account fully reads; a filter on another is empty.
    let keys: Vec<&str> = scopes
        .iter()
        .map(|s| s.key)
        .filter(|key| resource.is_empty() || *key == resource)
        .collect();
    let (entries, total_pages) = if keys.is_empty() {
        (Vec::new(), 1)
    } else {
        let mut query = history::Entity::find().filter(history::Column::ResourceKey.is_in(keys));
        if !object.is_empty() {
            query = query.filter(history::Column::ObjectPk.eq(object.as_str()));
        }
        if !action.is_empty() {
            query = query.filter(history::Column::Action.eq(action.as_str()));
        }
        if !username.is_empty() {
            query = query.filter(history::Column::Username.eq(username.as_str()));
        }
        let pages = query
            .order_by_desc(history::Column::Id)
            .paginate(&db, PAGE_SIZE);
        let found = match pages.num_pages().await {
            Ok(total) => pages.fetch_page(page - 1).await.map(|rows| (rows, total)),
            Err(e) => Err(e),
        };
        let (rows, total) = match found {
            Ok(found) => found,
            Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
        };
        let entries = rows
            .iter()
            .filter_map(|entry| {
                let scope = scopes.iter().find(|s| s.key == entry.resource_key)?;
                Some(row(entry, scope, &admin.paths))
            })
            .collect();
        (entries, total.max(1))
    };

    chrome(&mut req, &admin, &site, &user, "");
    crate::admin::trad::insert_admin_messages(&mut req.context, "history");
    put(
        &mut req,
        &HistoryContext {
            current_page: "history",
            history_url: admin.paths.history.clone(),
            entries,
            history_resources: scopes
                .iter()
                .map(|s| HistoryResource {
                    key: s.key,
                    title: s.title.clone(),
                })
                .collect(),
            filter_resource: resource,
            filter_object: object,
            filter_action: action,
            filter_user: username,
            page,
            total_pages,
        },
    )?;
    req.render(admin.config.templates.history.resolve())
}

pub(crate) async fn history_entry_view(
    Path(id): Path<i64>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let db = req.engine.db.clone();
    let not_found = AdminError::Refused(Refusal::new(RefusalReason::NotFound));
    let entry = match history::Entity::find_by_id(id).one(&db).await {
        Ok(Some(entry)) => entry,
        Ok(None) => return failure(not_found, &req, &admin).await,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    // An entry of a model this account doesn't fully read is refused like a
    // missing one: its existence tells nothing either.
    let scopes = site.history_scopes(&db, &user);
    let Some(scope) = scopes.iter().find(|s| s.key == entry.resource_key) else {
        return failure(not_found, &req, &admin).await;
    };

    chrome(&mut req, &admin, &site, &user, "");
    crate::admin::trad::insert_admin_messages(&mut req.context, "history");
    put(
        &mut req,
        &HistoryEntryContext {
            current_page: "history",
            history_url: admin.paths.history.clone(),
            entry: row(&entry, scope, &admin.paths),
            changes: changes(&entry, scope),
        },
    )?;
    req.render(admin.config.templates.history_diff.resolve())
}

#[cfg(test)]
mod tests {
    use super::row;
    use crate::admin::builder::registered::HistoryScope;
    use crate::admin::router::AdminPaths;
    use crate::auth::gate::rights::ReadableColumns;
    use crate::auth::history;

    fn entry(object_repr: &str) -> history::Model {
        history::Model {
            id: 1,
            resource_key: "users".into(),
            object_pk: "7".into(),
            object_repr: object_repr.into(),
            action: "update".into(),
            user_id: crate::utils::pk::Pk::default(),
            username: "carol".into(),
            created_at: chrono::Utc::now().naive_utc(),
            summary: None,
            batch_id: None,
        }
    }

    fn scope(label_readable: bool) -> HistoryScope {
        HistoryScope {
            key: "users",
            title: "Users".into(),
            readable: ReadableColumns::All,
            label_readable,
        }
    }

    /// The label is a column's value: shown only with that column's read
    /// right, else the row is named by its key alone.
    #[test]
    fn the_label_shows_only_when_its_column_is_readable() {
        let paths = AdminPaths::new("/admin");
        let shown = row(&entry("bob"), &scope(true), &paths);
        assert_eq!(shown.object_label.as_deref(), Some("bob"));
        let hidden = row(&entry("bob"), &scope(false), &paths);
        assert_eq!(hidden.object_label, None);
    }

    /// A label that is only the key repeats nothing worth showing.
    #[test]
    fn a_label_equal_to_the_key_is_not_shown() {
        let paths = AdminPaths::new("/admin");
        assert_eq!(row(&entry("7"), &scope(true), &paths).object_label, None);
    }
}
