//! `GET {admin}/{resource}/list`: one page of a model's list.
use crate::{
    admin::{
        builder::{
            list::ListQuery,
            model_admin::ListPage,
            registered::{AdminError, RegisteredModel, readable},
            site::AdminSite,
        },
        router::{
            account::summary::account_summaries,
            chrome,
            context::{
                Display, FilterOption, ListContext, ListEntry, Pager, ResourceLinks, ResourceRef,
                RowLinks, filter_clear_qs, put,
            },
            failure,
            path::AdminPaths,
            permission::rights::managed_groups,
            routes::AdminState,
            unknown_resource,
        },
        template::Page,
        translation::insert_admin_messages,
    },
    auth::{
        constants::resource_key::GROUPES, current_user::CurrentUser, gate::transfer::taken_by,
        permissions::rights::CrudAction,
    },
    errors::{AppError, ErrorContext},
    http::Request,
    schema::pk::Pk,
    translation::t,
    utils::aliases::{ADb, AppResult, StrMap},
};
use axum::{
    Extension,
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
};
use sea_orm::DbErr;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

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
    let mut page = match model
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
    log_page(&resource, &page, &query);

    let rights = model.rights(&user);
    let navigation = ListNavigation::of(&query, &page);
    let rows = std::mem::take(&mut page.rows);
    let mut entries = list_entries(rows, model, &rights, &admin.paths, &site)?;
    let rights_shortcut =
        match table_extras(&mut entries, model, &user, &req.engine.db, &admin.paths).await {
            Ok(shortcut) => shortcut,
            Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
        };
    let links = resource_links(model.key(), &rights, &page, rights_shortcut, &admin.paths);

    chrome(&mut req, &admin, &site, &user, model.key(), Page::List);
    insert_admin_messages(&mut req.context, "list");
    put(
        &mut req,
        &list_context(model, page, entries, navigation, links, &rights, &query),
    )?;

    let template = if req.headers.contains_key("hx-request") {
        admin.config.templates.htmx.resolve()
    } else {
        admin
            .config
            .templates
            .page_for(Page::List, Some(model.templates()))
    };
    req.render(template)
}

fn log_page(resource: &str, page: &ListPage, query: &ListQuery) {
    if let Some(level) = crate::log::log_level!(admin.list) {
        crate::runique_log!(
            level,
            resource = %resource,
            page = page.page,
            rows = page.rows.len(),
            total = page.total,
            filters = query.filters.len(),
            search = query.search.is_some(),
            "{}",
            t(runique_i18n::keys::admin::log::LIST)
        );
    }
}

/// Everything the page's links are built from: the query strings of the
/// filters, the sort and the size, and the three kinds of pagers (the list,
/// each filter group, the filter panel's size).
struct ListNavigation {
    page_count: u64,
    filter_qs: String,
    active_filters: HashMap<String, Vec<String>>,
    sort_dir: String,
    sort_dir_toggle: &'static str,
    size_qs: String,
    filter_nav: Vec<(String, String)>,
    filter_nav_qs: String,
    list_pager: Pager,
    filter_pagers: HashMap<&'static str, Pager>,
    filter_size_pager: Option<Pager>,
    filter_values: HashMap<&'static str, Vec<FilterOption>>,
    filter_clear: HashMap<&'static str, String>,
    return_qs: String,
}

impl ListNavigation {
    fn of(query: &ListQuery, page: &ListPage) -> Self {
        let page_count = page.total.div_ceil(page.page_size.max(1)).max(1);
        let filter_qs: String = query
            .filters
            .iter()
            .map(|(col, value)| {
                format!(
                    "&filter_{}={}",
                    urlencoding::encode(col),
                    urlencoding::encode(value)
                )
            })
            .collect();
        let mut active_filters: HashMap<String, Vec<String>> = HashMap::new();
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
        let filter_pagers: HashMap<&'static str, Pager> = page
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
        let filter_values: HashMap<&'static str, Vec<FilterOption>> = page
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
        let filter_clear: HashMap<&'static str, String> = page
            .filters
            .iter()
            .filter(|f| query.filters.iter().any(|(c, _)| c == f.column))
            .map(|f| (f.column, filter_clear_qs(&params, f.column)))
            .collect();
        let sort_dir_toggle = if sort_dir == "asc" { "desc" } else { "asc" };
        let return_qs = format!(
            "page={}&sort_by={}&sort_dir={}&search={}{filter_qs}{size_qs}{filter_nav_qs}",
            page.page,
            urlencoding::encode(query.sort_by.as_deref().unwrap_or_default()),
            urlencoding::encode(&sort_dir),
            urlencoding::encode(query.search.as_deref().unwrap_or_default()),
        );
        Self {
            page_count,
            filter_qs,
            active_filters,
            sort_dir,
            sort_dir_toggle,
            size_qs,
            filter_nav,
            filter_nav_qs,
            list_pager,
            filter_pagers,
            filter_size_pager,
            filter_values,
            filter_clear,
            return_qs,
        }
    }
}

/// Each row with its links. Offered on the list's rights; the gate decides
/// on the row's page.
fn list_entries(
    rows: Vec<Value>,
    model: &dyn RegisteredModel,
    rights: &CrudAction,
    paths: &AdminPaths,
    site: &AdminSite,
) -> AppResult<Vec<ListEntry>> {
    let key = model.key();
    let pk = model.pk_column();
    let can_edit = rights.can_update || rights.can_update_own;
    let can_delete = rights.can_delete || rights.can_delete_own;
    let mut entries = Vec::with_capacity(rows.len());
    for values in rows {
        let Some(id) = values.get(pk).and_then(|value| match value {
            Value::String(id) => Some(id.clone()),
            Value::Number(id) => Some(id.to_string()),
            _ => None,
        }) else {
            return Err(Box::new(AppError::new(ErrorContext::generic(
                StatusCode::INTERNAL_SERVER_ERROR,
                &crate::translation::t(runique_i18n::keys::admin::common::error::ROW_WITHOUT_KEY),
            ))));
        };
        let links = RowLinks {
            detail: paths.detail_of(site, key, &id),
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
    Ok(entries)
}

/// What only some tables add: each account's summary, each group's taker,
/// and on the groups the shortcut to the rights grid.
async fn table_extras(
    entries: &mut [ListEntry],
    model: &dyn RegisteredModel,
    user: &CurrentUser,
    db: &ADb,
    paths: &AdminPaths,
) -> Result<Option<String>, DbErr> {
    if model.is_account_table() {
        let ids: Vec<Pk> = entries.iter().filter_map(|e| e.id.parse().ok()).collect();
        let mut summaries = account_summaries(db, &ids).await?;
        for entry in entries.iter_mut() {
            entry.account = entry.id.parse().ok().and_then(|id| summaries.remove(&id));
        }
    }
    if model.key() != GROUPES {
        return Ok(None);
    }
    let ids: Vec<i32> = entries.iter().filter_map(|e| e.id.parse().ok()).collect();
    let names = taken_by(&ids, db).await?;
    for entry in entries.iter_mut() {
        entry.taken_by = entry.id.parse().ok().and_then(|id| names.get(&id).cloned());
    }
    let groups = managed_groups(user, db).await?;
    Ok((!groups.is_empty()).then(|| paths.group_select.clone()))
}

fn resource_links(
    key: &'static str,
    rights: &CrudAction,
    page: &ListPage,
    rights_shortcut: Option<String>,
    paths: &AdminPaths,
) -> ResourceLinks {
    let can_delete = rights.can_delete || rights.can_delete_own;
    let bulk_offered = !page.actions.is_empty() || can_delete;
    ResourceLinks {
        list: paths.list_of(key),
        create: rights.can_create.then(|| paths.create_of(key)),
        bulk: bulk_offered.then(|| paths.bulk_of(key)),
        rights: rights_shortcut,
    }
}

fn list_context(
    model: &dyn RegisteredModel,
    page: ListPage,
    entries: Vec<ListEntry>,
    navigation: ListNavigation,
    resource_links: ResourceLinks,
    rights: &CrudAction,
    query: &ListQuery,
) -> ListContext {
    ListContext {
        resource: ResourceRef {
            display: Display {
                list_filter: page
                    .filters
                    .iter()
                    .map(|f| (f.column.to_string(), readable(f.column)))
                    .collect(),
            },
            ..ResourceRef::new(model.key(), model.title())
        },
        filter_values: navigation.filter_values,
        filter_clear: navigation.filter_clear,
        resource_links,
        current_page: "list",
        entries,
        visible_columns: page.columns,
        sortable_columns: page.sortable,
        column_labels: StrMap::new(),
        rich_fields: Vec::new(),
        group_actions: page.actions,
        return_qs: navigation.return_qs,
        active_filters: navigation.active_filters,
        filter_qs: navigation.filter_qs,
        search: query.search.clone().unwrap_or_default(),
        sort_by: query.sort_by.clone().unwrap_or_default(),
        sort_dir: navigation.sort_dir,
        sort_dir_toggle: navigation.sort_dir_toggle,
        total: page.total,
        page: page.page,
        page_count: navigation.page_count,
        list_pager: navigation.list_pager,
        filter_pagers: navigation.filter_pagers,
        filter_size_pager: navigation.filter_size_pager,
        bulk_delete: rights.can_delete || rights.can_delete_own,
        page_size: page.page_size,
        size_qs: navigation.size_qs,
        filter_nav_qs: navigation.filter_nav_qs,
        filter_nav: navigation.filter_nav,
    }
}
