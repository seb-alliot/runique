//! `GET {admin}/{resource}/list`: one page of a model's list.
use super::context::{Display, ListContext, ResourceRef, put};
use super::{chrome, failure, unknown_resource};
use crate::admin::builder::list::ListQuery;
use crate::admin::builder::registered::readable;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::AdminView;
use crate::admin::router::admin_router::AdminState;
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
    let mut page = match model
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
    let detail_route = site
        .overridden_path(key, AdminView::Detail)
        .map(str::to_owned);
    if let Some(detail_route) = detail_route {
        let pk = model.pk_column();
        for row in &mut page.rows {
            let Some(id) = row.get(pk).and_then(|value| match value {
                serde_json::Value::String(id) => Some(id.clone()),
                serde_json::Value::Number(id) => Some(id.to_string()),
                _ => None,
            }) else {
                return Err(Box::new(AppError::new(ErrorContext::generic(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "admin list row is missing a serializable primary key",
                ))));
            };
            let Some(fields) = row.as_object_mut() else {
                return Err(Box::new(AppError::new(ErrorContext::generic(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "admin list row is not an object",
                ))));
            };
            let encoded_id = urlencoding::encode(&id);
            fields.insert(
                "detail_url".to_string(),
                serde_json::Value::String(detail_route.replace("{id}", &encoded_id)),
            );
        }
    }

    chrome(&mut req, &admin, &site, &user, key);
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
            resource_base: format!("{}/{key}", admin.config.prefix),
            current_page: "list",
            entries: page.rows,
            visible_columns: page.columns,
            column_labels: StrMap::new(),
            rich_fields: Vec::new(),
            can_bulk: !page.actions.is_empty() || can_delete,
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
            can_view: true,
            can_create: rights.can_create,
            can_update: false,
            can_edit,
            can_delete,
        },
    )?;

    let htmx = req.headers.contains_key("hx-request");
    let template = if htmx {
        admin.config.templates.htmx.resolve()
    } else {
        admin.config.templates.list.resolve()
    };
    req.render(template)
}
