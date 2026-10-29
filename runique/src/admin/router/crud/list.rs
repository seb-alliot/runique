//! `GET {admin}/{resource}/list`: one page of a model's list.
use crate::admin::builder::list::ListQuery;
use crate::admin::builder::registered::readable;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{
    Display, ListContext, ListEntry, ResourceLinks, ResourceRef, RowLinks, put,
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
    Query(params): Query<StrMap>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let query = ListQuery::from_params(params, admin.config.search_max_len);
    let page = match model
        .list(&req.engine.db, &user, &query, admin.config.page_size)
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
    let active_filters: StrMap = query.filters.iter().cloned().collect();
    let sort_dir = query.sort_dir.clone().unwrap_or_else(|| "asc".to_string());
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
        });
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
            filter_values: page
                .filters
                .into_iter()
                .map(|f| (f.column, f.values))
                .collect(),
            resource_links: ResourceLinks {
                list: paths.list_of(key),
                create: rights.can_create.then(|| paths.create_of(key)),
                bulk: bulk_offered.then(|| paths.bulk_of(key)),
                rights: rights_shortcut,
            },
            current_page: "list",
            entries,
            visible_columns: page.columns,
            column_labels: StrMap::new(),
            rich_fields: Vec::new(),
            group_actions: page.actions,
            return_qs: format!(
                "page={}&sort_by={}&sort_dir={}&search={}{}",
                page.page,
                urlencoding::encode(query.sort_by.as_deref().unwrap_or_default()),
                urlencoding::encode(&sort_dir),
                urlencoding::encode(query.search.as_deref().unwrap_or_default()),
                filter_qs
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
            has_prev: page.page > 1,
            has_next: page.page < page_count,
            prev_page: page.page.saturating_sub(1).max(1),
            next_page: page.page + 1,
            bulk_delete: can_delete,
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
