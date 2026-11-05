//! `GET {admin}/{resource}/list`: one page of a model's list.
use crate::admin::builder::list::ListQuery;
use crate::admin::builder::registered::readable;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{
    Display, FilterOption, ListContext, ListEntry, Pager, ResourceLinks, ResourceRef, RowLinks,
    filter_clear_qs, put,
};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::aliases::{AppResult, StrMap};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use std::sync::Arc;

pub(crate) async fn list_view(
    Path(resource): Path<String>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    // A list of pairs: a filter sends one name several times.
    Query(params): Query<Vec<(String, String)>>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let query = ListQuery::from_params(params, admin.config.search_max_len);
    let page = match model
        .list(
            &req.engine.db,
            &user,
            &query,
            admin.config.page_size,
            admin.config.filter_size,
        )
        .await
    {
        Ok(page) => page,
        Err(error) => return failure(error, &req, &admin).await,
    };

    let page_count = page.total.div_ceil(page.page_size.max(1)).max(1);
    let filter_qs: String = query
        .filters
        .iter()
        .map(|(col, value)| format!("&filter_{col}={}", urlencoding::encode(value)))
        .collect();
    let mut active_filters: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (col, value) in &query.filters {
        active_filters
            .entry(col.clone())
            .or_default()
            .push(value.clone());
    }
    let sort_dir = query.sort_dir.clone().unwrap_or_else(|| "asc".to_string());
    let size_qs = query
        .per_page
        .map(|n| format!("&per_page={n}"))
        .unwrap_or_default();
    // The panel's size from the URL (a size the model or site set isn't pinned
    // in the links, like `per_page`), and the groups moved, read from the
    // list's own filter columns only: a name made up in the URL is never sent back.
    let filter_nav: Vec<(String, String)> = query
        .filter_size
        .map(|n| ("fsize".to_string(), n.to_string()))
        .into_iter()
        .chain(
            page.filters
                .iter()
                .filter(|f| f.page > 1)
                .map(|f| (format!("fpage_{}", f.column), f.page.to_string())),
        )
        .collect();
    let filter_nav_qs: String = filter_nav
        .iter()
        .map(|(k, v)| format!("&{k}={v}"))
        .collect();
    // The page's whole query, from which every pager builds its links.
    let mut params: Vec<(String, String)> = Vec::new();
    if page.page > 1 {
        params.push(("page".into(), page.page.to_string()));
    }
    if let Some(n) = query.per_page {
        params.push(("per_page".into(), n.to_string()));
    }
    if let Some(sort_by) = &query.sort_by {
        params.push(("sort_by".into(), sort_by.clone()));
        params.push(("sort_dir".into(), sort_dir.clone()));
    }
    if let Some(search) = &query.search {
        params.push(("search".into(), search.clone()));
    }
    for (col, value) in &query.filters {
        params.push((format!("filter_{col}"), value.clone()));
    }
    params.extend(filter_nav.iter().cloned());
    let list_pager = Pager::new(
        &params,
        "page",
        "per_page",
        page.page,
        Some(page_count),
        page.page < page_count,
        page.page_size,
    );
    let filter_pagers: std::collections::HashMap<&'static str, Pager> = page
        .filters
        .iter()
        .map(|f| {
            let pager = Pager::new(
                &params,
                &format!("fpage_{}", f.column),
                "fsize",
                f.page,
                None,
                f.has_next,
                f.size,
            );
            (f.column, pager)
        })
        .collect();
    // The panel's one selector. A new size restarts every group from its first
    // page, so none is kept; no parameter is named `fpage_`, nothing to move.
    let unpaged: Vec<(String, String)> = params
        .iter()
        .filter(|(k, _)| !k.starts_with("fpage_"))
        .cloned()
        .collect();
    let filter_size_pager = page
        .filters
        .first()
        .map(|f| Pager::new(&unpaged, "fpage_", "fsize", 1, None, false, f.size));
    let filter_values: std::collections::HashMap<&'static str, Vec<FilterOption>> = page
        .filters
        .iter()
        .map(|f| {
            let applied_elsewhere = query
                .filters
                .iter()
                .filter(|(c, v)| c == f.column && !f.values.contains(v))
                .map(|(_, v)| v.clone());
            let options = applied_elsewhere
                .chain(f.values.iter().cloned())
                .map(|value| FilterOption::new(&params, f.column, value))
                .collect();
            (f.column, options)
        })
        .collect();
    let filter_clear: std::collections::HashMap<&'static str, String> = page
        .filters
        .iter()
        .filter(|f| query.filters.iter().any(|(c, _)| c == f.column))
        .map(|f| (f.column, filter_clear_qs(&params, f.column)))
        .collect();
    let sort_dir_toggle = if sort_dir == "asc" { "desc" } else { "asc" };
    let key = model.key();
    let rights = model.rights(&user);
    let can_edit = rights.can_update || rights.can_update_own;
    let can_delete = rights.can_delete || rights.can_delete_own;
    let paths = &admin.paths;
    let pk = model.pk_column();
    let mut entries = Vec::with_capacity(page.rows.len());
    for values in page.rows {
        let Some(id) = values.get(pk).and_then(|value| match value {
            serde_json::Value::String(id) => Some(id.clone()),
            serde_json::Value::Number(id) => Some(id.to_string()),
            _ => None,
        }) else {
            return Err(Box::new(AppError::new(ErrorContext::generic(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "admin list row is missing a serializable primary key",
            ))));
        };
        // Offered on the list's rights; the gate decides on the row's page.
        let links = RowLinks {
            detail: paths.detail_of(&site, key, &id),
            edit: can_edit.then(|| paths.edit_of(key, &id)),
            delete: can_delete.then(|| paths.delete_of(key, &id)),
        };
        entries.push(ListEntry {
            id,
            values,
            links,
            taken_by: None,
            account: None,
        });
    }
    if model.is_account_table() {
        let ids: Vec<crate::utils::pk::Pk> =
            entries.iter().filter_map(|e| e.id.parse().ok()).collect();
        match crate::admin::router::account::summary::account_summaries(&req.engine.db, &ids).await
        {
            Ok(mut summaries) => {
                for entry in &mut entries {
                    entry.account = entry.id.parse().ok().and_then(|id| summaries.remove(&id));
                }
            }
            Err(e) => {
                return failure(
                    crate::admin::builder::registered::AdminError::Db(e),
                    &req,
                    &admin,
                )
                .await;
            }
        }
    }
    if key == crate::utils::constante::resource_key::GROUPES {
        let ids: Vec<i32> = entries.iter().filter_map(|e| e.id.parse().ok()).collect();
        match crate::auth::gate::transfer::taken_by(&ids, &req.engine.db).await {
            Ok(names) => {
                for entry in &mut entries {
                    entry.taken_by = entry.id.parse().ok().and_then(|id| names.get(&id).cloned());
                }
            }
            Err(e) => {
                return failure(
                    crate::admin::builder::registered::AdminError::Db(e),
                    &req,
                    &admin,
                )
                .await;
            }
        }
    }
    let bulk_offered = !page.actions.is_empty() || can_delete;
    let rights_shortcut = if key == crate::utils::constante::resource_key::GROUPES {
        match crate::admin::router::permission::rights::managed_groups(&user, &req.engine.db).await
        {
            Ok(groups) => (!groups.is_empty()).then(|| paths.group_select.clone()),
            Err(e) => {
                return failure(
                    crate::admin::builder::registered::AdminError::Db(e),
                    &req,
                    &admin,
                )
                .await;
            }
        }
    } else {
        None
    };

    chrome(
        &mut req,
        &admin,
        &site,
        &user,
        key,
        crate::admin::template::Page::List,
    );
    insert_admin_messages(&mut req.context, "list");
    put(
        &mut req,
        &ListContext {
            resource: ResourceRef {
                display: Display {
                    list_filter: page
                        .filters
                        .iter()
                        .map(|f| (f.column.to_string(), readable(f.column)))
                        .collect(),
                },
                ..ResourceRef::new(key, model.title())
            },
            filter_values,
            filter_clear,
            resource_links: ResourceLinks {
                list: paths.list_of(key),
                create: rights.can_create.then(|| paths.create_of(key)),
                bulk: bulk_offered.then(|| paths.bulk_of(key)),
                rights: rights_shortcut,
            },
            current_page: "list",
            entries,
            visible_columns: page.columns,
            sortable_columns: page.sortable,
            column_labels: StrMap::new(),
            rich_fields: Vec::new(),
            group_actions: page.actions,
            return_qs: format!(
                "page={}&sort_by={}&sort_dir={}&search={}{filter_qs}{size_qs}{filter_nav_qs}",
                page.page,
                urlencoding::encode(query.sort_by.as_deref().unwrap_or_default()),
                urlencoding::encode(&sort_dir),
                urlencoding::encode(query.search.as_deref().unwrap_or_default()),
            ),
            active_filters,
            filter_qs,
            search: query.search.clone().unwrap_or_default(),
            sort_by: query.sort_by.clone().unwrap_or_default(),
            sort_dir,
            sort_dir_toggle,
            total: page.total,
            page: page.page,
            page_count,
            list_pager,
            filter_pagers,
            filter_size_pager,
            bulk_delete: can_delete,
            page_size: page.page_size,
            size_qs,
            filter_nav_qs,
            filter_nav,
        },
    )?;

    let htmx = req.headers.contains_key("hx-request");
    let template = if htmx {
        admin.config.templates.htmx.resolve()
    } else {
        admin
            .config
            .templates
            .page_for(crate::admin::template::Page::List, Some(model.templates()))
    };
    req.render(template)
}
