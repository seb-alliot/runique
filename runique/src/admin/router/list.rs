//! `GET {admin}/{resource}/list`: one page of a model's list.
use super::{chrome, failure, unknown_resource};
use crate::admin::builder::list::ListQuery;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::context::template::Request;
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

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "list");
    req = req
        .insert(
            "resource",
            serde_json::json!({ "key": key, "title": model.title(), "display": { "list_filter": [] } }),
        )
        .insert("resource_base", format!("{}/{key}", admin.config.prefix))
        .insert("current_page", "list")
        .insert("entries", &page.rows)
        .insert("visible_columns", &page.columns)
        .insert("column_labels", StrMap::new())
        .insert("rich_fields", Vec::<String>::new())
        .insert("group_actions", Vec::<String>::new())
        .insert("active_filters", &active_filters)
        .insert("filter_qs", &filter_qs)
        .insert("search", query.search.clone().unwrap_or_default())
        .insert("sort_by", query.sort_by.clone().unwrap_or_default())
        .insert("sort_dir", &sort_dir)
        .insert("sort_dir_toggle", sort_dir_toggle)
        .insert("total", page.total)
        .insert("page", page.page)
        .insert("page_count", page_count)
        .insert("has_prev", page.page > 1)
        .insert("has_next", page.page < page_count)
        .insert("prev_page", page.page.saturating_sub(1).max(1))
        .insert("next_page", page.page + 1)
        // Reading only, until writing goes through the gate: no action is offered.
        .insert("can_view", true)
        .insert("can_create", false)
        .insert("can_update", false)
        .insert("can_edit", false)
        .insert("can_delete", false)
        .insert("can_bulk", false);

    let htmx = req.headers.contains_key("hx-request");
    let template = if htmx {
        admin.config.templates.htmx.resolve()
    } else {
        admin.config.templates.list.resolve()
    };
    req.render(template)
}
