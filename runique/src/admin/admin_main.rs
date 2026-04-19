//! Generic CRUD handler for the admin interface.
//!
//! Covered routes:
//! - `GET/POST /admin/{resource}/{action}` → [`admin_get`] / [`admin_post`]
//! - `GET/POST /admin/{resource}/{id}/{action}` → [`admin_get_id`] / [`admin_post_id`]
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::forms::prisme::aegis;
use crate::utils::{
    aliases::{ARuniqueConfig, AppResult, StrMap},
    constante::admin_context::{
        common as ctx_common, create as ctx_create, detail as ctx_detail, edit as ctx_edit,
        list as list_ctx,
    },
    session_key::session::CSRF_TOKEN_KEY,
    trad::{current_lang, t},
};
use crate::{
    admin::{
        AdminRegistry,
        config::AdminConfig,
        helper::resource_entry::{ListParams, ResourceEntry, SortDir},
        resource::ColumnFilter,
        trad::insert_admin_messages,
    },
    utils::admin_context::list::{PAGE, SORT_BY, SORT_DIR},
};
use axum::{
    Extension,
    body::Body,
    extract::{FromRequest, Path, Query},
    http::{Request as HttpRequest, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use subtle::ConstantTimeEq;

fn is_unique_violation(e: &sea_orm::DbErr) -> bool {
    let msg = e.to_string();
    msg.contains("unique") || msg.contains("UNIQUE") || msg.contains("Duplicate")
}

// ─── ListQuery — admin list view parameters ─────────────────────

struct ListQuery {
    page: u64,
    sort_by: Option<String>,
    sort_dir: SortDir,
    search: Option<String>,
    column_filters: Vec<(String, String)>,
    filter_pages: HashMap<String, u64>,
}

// ─── AdminBody Extractor ─────────────────────────────────────
//
// Encapsulates aegis to automatically handle multipart/form-data
// AND application/x-www-form-urlencoded in admin POST handlers.
// Replaces axum::extract::Form<StrMap> which rejected multipart.

pub struct AdminBody(StrMap);

impl<S: Send + Sync> FromRequest<S> for AdminBody {
    type Rejection = Response;

    async fn from_request(req: HttpRequest<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let config = req
            .extensions()
            .get::<ARuniqueConfig>()
            .cloned()
            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "missing config").into_response())?;

        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let parsed = aegis(req, state, config, &content_type).await?;

        // StrVecMap → StrMap (multi-values joined by comma, like Prisme)
        // Exception : csrf_token takes the 1st value only (the form
        // may send two if {% csrf %} and form_fields.html coexist).
        let body = parsed
            .into_iter()
            .map(|(k, v)| {
                if k == CSRF_TOKEN_KEY {
                    (k, v.into_iter().next().unwrap_or_default())
                } else {
                    (k, v.join(","))
                }
            })
            .collect();
        Ok(AdminBody(body))
    }
}

// ─── Shared state ────────────────────────────────────────────

#[derive(Clone)]
pub struct PrototypeAdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
}

// ─── Axum entry points ────────────────────────────────────

/// GET /admin/{resource}/{action}  (list, create)
pub async fn admin_get(
    mut req: Request,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    headers: axum::http::HeaderMap,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry, &current_user);

    match action.as_str() {
        "list" => {
            if !current_user.can_access_resource(&resource_key) {
                return Ok(permission_denied_dashboard(&req.notices, &state.config.prefix).await);
            }
            let page = params
                .get(PAGE)
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(1)
                .max(1);
            let sort_by = params.get(SORT_BY).filter(|s| !s.is_empty()).cloned();
            let sort_dir = match params.get(SORT_DIR).map(String::as_str) {
                Some("desc") => SortDir::Desc,
                _ => SortDir::Asc,
            };
            let search = params.get("search").filter(|s| !s.is_empty()).cloned();
            let column_filters: Vec<(String, String)> = params
                .iter()
                .filter_map(|(k, v)| {
                    k.strip_prefix("filter_")
                        .filter(|_| !v.is_empty())
                        .map(|col| (col.to_string(), v.clone()))
                })
                .collect();
            let filter_pages: HashMap<String, u64> = params
                .iter()
                .filter_map(|(k, v)| {
                    let col = k.strip_prefix("fp_")?;
                    let page = v.parse::<u64>().ok()?;
                    Some((col.to_string(), page))
                })
                .collect();
            let query = ListQuery {
                page,
                sort_by,
                sort_dir,
                search,
                column_filters,
                filter_pages,
            };
            let is_htmx = headers.contains_key("hx-request");
            handle_list(&mut req, entry, &state, query, &current_user, is_htmx).await
        }
        "create" => {
            if !current_user.can_access_resource(&resource_key) {
                return Ok(permission_denied_dashboard(&req.notices, &state.config.prefix).await);
            }
            if !check_write_access(&current_user, &resource_key) {
                return Ok(
                    permission_denied(&req.notices, &state.config.prefix, &resource_key).await,
                );
            }
            handle_create_get(&mut req, entry, &state).await
        }
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// POST /admin/{resource}/{action}  (create)
#[allow(private_interfaces)]
pub async fn admin_post(
    mut req: Request,
    headers: axum::http::HeaderMap,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    AdminBody(body): AdminBody,
) -> AppResult<Response> {
    tracing::info!("form data at validation {:?}", body);
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;
    if !check_write_access(&current_user, &resource_key) {
        return Ok(permission_denied(&req.notices, &state.config.prefix, &resource_key).await);
    }
    match action.as_str() {
        "create" => handle_create_post(&mut req, entry, body, &headers, &state).await,
        "bulk" => handle_bulk_action(&mut req, entry, body, &state, &resource_key).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// GET /admin/{resource}/{id}/{action}  (detail, edit, delete)
pub async fn admin_get_id(
    mut req: Request,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());

    if !current_user.can_access_resource(&resource_key) {
        return Ok(permission_denied_dashboard(&req.notices, &state.config.prefix).await);
    }
    if !check_write_access(&current_user, &resource_key)
        && matches!(action.as_str(), "edit" | "delete")
    {
        return Ok(permission_denied(&req.notices, &state.config.prefix, &resource_key).await);
    }
    match action.as_str() {
        "detail" => handle_detail(&mut req, entry, id, &state).await,
        "edit" => handle_edit_get(&mut req, entry, id, &state).await,
        "delete" => handle_delete_get(&mut req, entry, id, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// POST /admin/{resource}/{id}/{action}  (edit, delete)
#[allow(private_interfaces)]
pub async fn admin_post_id(
    mut req: Request,
    headers: axum::http::HeaderMap,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    AdminBody(body): AdminBody,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;
    if !check_write_access(&current_user, &resource_key) {
        return Ok(permission_denied(&req.notices, &state.config.prefix, &resource_key).await);
    }

    match action.as_str() {
        "edit" => handle_edit_post(&mut req, entry, id, body, &state).await,
        "delete" => handle_delete_post(&mut req, entry, id, &state).await,
        "reset-password" => handle_reset_password(&mut req, entry, id, &headers, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

// ─── Internal functions ──────────────────────────────────────

fn inject_context(
    req: &mut Request,
    state: &PrototypeAdminState,
    entry: &ResourceEntry,
    current_user: &CurrentUser,
) {
    for item in ["list", "create", "edit", "detail", "delete", "base"] {
        insert_admin_messages(&mut req.context, item);
    }

    req.context
        .insert(ctx_common::SITE_TITLE, &state.config.site_title);
    req.context
        .insert(ctx_common::SITE_URL, &state.config.site_url);
    req.context.insert(ctx_common::RESOURCE_KEY, entry.meta.key);
    req.context
        .insert(ctx_common::CURRENT_RESOURCE, entry.meta.key);
    req.context.insert(ctx_common::RESOURCE, &entry.meta);
    req.context.insert("group_actions", &entry.group_actions);

    // Filters visible resources based on user's scoped rights.
    // - is_superuser → sees everything
    // - other resources → user must have can_read on resource_key
    let visible_resources: Vec<_> = state
        .registry
        .all()
        .filter(|e| {
            if current_user.is_superuser {
                return true;
            }
            current_user.can_access_resource(e.meta.key)
        })
        .map(|e| &e.meta)
        .collect();
    req.context
        .insert(ctx_common::RESOURCES, &visible_resources);

    for (k, v) in &entry.meta.extra_context {
        req.context.insert(k, v);
    }
}

/// Checks if the user has write access to the resource.
/// Superuser bypass. Otherwise, checks DB permissions.
fn check_write_access(user: &CurrentUser, resource_key: &str) -> bool {
    user.is_superuser
        || user.permissions_effectives().iter().any(|d| {
            d.resource_key == resource_key && (d.can_create || d.can_update || d.can_delete)
        })
}

/// Redirects to the resource list with a permission error message.
async fn permission_denied(
    notices: &crate::flash::flash_manager::Message,
    prefix: &str,
    resource_key: &str,
) -> Response {
    notices
        .error(t("admin.access.insufficient_rights").to_string())
        .await;
    Redirect::to(&format!(
        "{}/{}/list",
        prefix.trim_end_matches('/'),
        resource_key
    ))
    .into_response()
}

/// Redirects to the admin dashboard with a permission error message.
async fn permission_denied_dashboard(
    notices: &crate::flash::flash_manager::Message,
    prefix: &str,
) -> Response {
    notices
        .error(t("admin.access.insufficient_rights").to_string())
        .await;
    Redirect::to(&format!("{}/", prefix.trim_end_matches('/'))).into_response()
}

/// Checks the CSRF token from the form body.
/// The middleware delegates form validation to Prisme — we do it manually here.
fn check_csrf(body: &StrMap, session_token: &str) -> AppResult<()> {
    let valid = body
        .get(CSRF_TOKEN_KEY)
        .map(|s| {
            if let Ok(unmasked) = crate::utils::csrf::unmask_csrf_token(s) {
                bool::from(unmasked.as_bytes().ct_eq(session_token.as_bytes()))
            } else {
                bool::from(s.as_bytes().ct_eq(session_token.as_bytes()))
            }
        })
        .unwrap_or(false);
    if !valid {
        return Err(Box::new(AppError::new(ErrorContext::generic(
            StatusCode::FORBIDDEN,
            t("csrf.invalid_or_missing").as_ref(),
        ))));
    }
    Ok(())
}

/// Converts a `Value::Object` to `StrMap` to pre-fill a form.
fn value_to_strmap(v: Value) -> StrMap {
    let mut map = StrMap::new();
    if let Value::Object(obj) = v {
        for (k, v) in obj {
            let s = match v {
                Value::Null => String::new(),
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                // Arrays and nested objects cannot pre-fill a flat field
                Value::Array(_) | Value::Object(_) => continue,
            };
            map.insert(k, s);
        }
    }
    map
}

// ─── Bulk actions ────────────────────────────────────────────

fn parse_bulk_ids(body: &StrMap) -> Vec<String> {
    body.get("ids")
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

async fn handle_bulk_action(
    req: &mut Request,
    entry: &ResourceEntry,
    body: StrMap,
    state: &PrototypeAdminState,
    resource_key: &str,
) -> AppResult<Response> {
    let ids = parse_bulk_ids(&body);
    let list_url = format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        resource_key
    );

    if ids.is_empty() {
        req.notices
            .warning(t("admin.bulk.no_selection").to_string())
            .await;
        return Ok(Redirect::to(&list_url).into_response());
    }

    match body.get("bulk_action").map(String::as_str).unwrap_or("") {
        "delete" => handle_bulk_delete(req, entry, ids, state, resource_key).await,
        "group_set" => handle_group_set(req, entry, ids, body, state, resource_key).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown bulk action",
        )))),
    }
}

async fn handle_group_set(
    req: &mut Request,
    entry: &ResourceEntry,
    ids: Vec<String>,
    body: StrMap,
    state: &PrototypeAdminState,
    resource_key: &str,
) -> AppResult<Response> {
    let list_url = format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        resource_key
    );

    let updates: StrMap = body
        .iter()
        .filter_map(|(k, v)| {
            k.strip_prefix("ga_")
                .filter(|_| !v.is_empty())
                .map(|field| (field.to_string(), v.clone()))
        })
        .collect();

    if updates.is_empty() {
        req.notices
            .warning(t("admin.bulk.no_field_selected").to_string())
            .await;
        return Ok(Redirect::to(&list_url).into_response());
    }

    let update_fn = entry
        .partial_update_fn
        .as_ref()
        .or(entry.update_fn.as_ref())
        .ok_or_else(|| {
            Box::new(AppError::new(ErrorContext::not_found(
                t("admin.delete.not_found").as_ref(),
            )))
        })?;

    let count = ids.len();
    for id in ids {
        update_fn(req.engine.db.clone(), id, updates.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    }

    req.notices
        .success(format!("{count} {}", t("admin.bulk.update_success")))
        .await;
    Ok(Redirect::to(&list_url).into_response())
}

async fn handle_bulk_delete(
    req: &mut Request,
    entry: &ResourceEntry,
    ids: Vec<String>,
    state: &PrototypeAdminState,
    resource_key: &str,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    let count = ids.len();
    for id in ids {
        delete_fn(req.engine.db.clone(), id)
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    }

    req.notices
        .success(format!("{count} {}", t("admin.bulk.delete_success")))
        .await;
    Ok(Redirect::to(&format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        resource_key
    ))
    .into_response())
}

async fn handle_list(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &PrototypeAdminState,
    query: ListQuery,
    current_user: &CurrentUser,
    is_htmx: bool,
) -> AppResult<Response> {
    if !current_user.can_access_resource(entry.meta.key) {
        return Ok(permission_denied_dashboard(&req.notices, &state.config.prefix).await);
    }
    inject_context(req, state, entry, current_user);
    let ListQuery {
        page,
        sort_by,
        sort_dir,
        search,
        column_filters,
        filter_pages,
    } = query;
    let page_size = state.config.page_size;
    let offset = page.saturating_sub(1).saturating_mul(page_size);
    let list_params = ListParams {
        offset,
        limit: page_size,
        sort_by: sort_by.clone(),
        sort_dir: sort_dir.clone(),
        search: search.clone(),
        column_filters: column_filters.clone(),
    };

    let (entries_result, count_result, filter_result) = tokio::join!(
        async {
            match &entry.list_fn {
                Some(f) => f(req.engine.db.clone(), list_params).await,
                None => Ok(Vec::new()),
            }
        },
        async {
            match &entry.count_fn {
                Some(f) => f(req.engine.db.clone(), search.clone()).await,
                None => Ok(0u64),
            }
        },
        async {
            match &entry.filter_fn {
                Some(f) => f(req.engine.db.clone(), filter_pages.clone())
                    .await
                    .unwrap_or_else(|e| {
                        if let Some(level) = crate::utils::runique_log::get_log().filter_fn {
                            crate::runique_log!(level, resource = entry.meta.key, error = %e, "filter_fn failed — list returned without sidebar filters");
                        }
                        HashMap::new()
                    }),
                None => HashMap::new(),
            }
        }
    );
    let entries = entries_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    let count = count_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    // Separate distinct values and totals by column
    let filter_values: HashMap<String, Vec<String>> = filter_result
        .iter()
        .map(|(k, (vals, _))| (k.clone(), vals.clone()))
        .collect();
    let filter_totals: HashMap<String, u64> = filter_result
        .into_iter()
        .map(|(k, (_, total))| (k, total))
        .collect();
    // If count_fn is absent, estimates the total from the current page (avoids broken pagination)
    let total = if entry.count_fn.is_some() {
        count
    } else {
        offset.saturating_add(entries.len() as u64)
    };

    let page_count = total.div_ceil(page_size);
    let page = page.min(page_count.max(1));

    // Visible columns: all except id/password, filtered by DisplayConfig
    let all_cols: Vec<String> = entries
        .first()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.keys()
                .filter(|k| *k != "id" && !k.starts_with("password"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let (visible_columns, mut column_labels): (Vec<String>, HashMap<String, String>) =
        match &entry.meta.display.columns {
            ColumnFilter::All => (all_cols, HashMap::new()),
            ColumnFilter::Include(cols) => {
                let filtered: Vec<(String, String)> = cols
                    .iter()
                    .filter(|(c, _)| all_cols.contains(c))
                    .cloned()
                    .collect();
                let labels = filtered
                    .iter()
                    .map(|(c, l)| (c.clone(), l.clone()))
                    .collect();
                (filtered.into_iter().map(|(c, _)| c).collect(), labels)
            }
            ColumnFilter::Exclude(excluded) => (
                all_cols
                    .into_iter()
                    .filter(|c| !excluded.contains(c))
                    .collect(),
                HashMap::new(),
            ),
        };

    // Auto-populate column_labels from i18n keys "permission.col.{col}"
    // for columns without an explicit label.
    for col in &visible_columns {
        if !column_labels.contains_key(col) {
            let key = format!("permission.col.{col}");
            let translated = t(&key);
            if translated != key.as_str() {
                column_labels.insert(col.clone(), translated.into_owned());
            }
        }
    }

    // Whitelist validation: sort_by must be a visible column or "id"
    let safe_sort_by = sort_by
        .filter(|s| s == "id" || visible_columns.contains(s))
        .unwrap_or_default();

    // active_filters: all list_filter columns initialized to "" then overwritten if active
    // (avoids Tera "key not found" error when accessing active_filters[col])
    let mut active_filters: HashMap<String, String> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, _)| (col.clone(), String::new()))
        .collect();
    for (col, val) in &column_filters {
        active_filters.insert(col.clone(), val.clone());
    }

    // filter_qs: query string added to pagination / sort / search links
    // Includes active filters AND sidebar pages (fp_*) to preserve them
    let filter_qs: String = {
        let mut parts: Vec<String> = column_filters
            .iter()
            .map(|(col, val)| format!("&filter_{}={}", col, urlencoding::encode(val)))
            .collect();
        for (col, page) in &filter_pages {
            if *page > 0 {
                parts.push(format!("&fp_{}={}", col, page));
            }
        }
        parts.concat()
    };

    // filter_meta: pre-calculated prev/next QS by column for sidebar pagination
    let base_qs: Vec<String> = {
        let mut parts = vec![];
        if !safe_sort_by.is_empty() {
            parts.push(format!("sort_by={}", safe_sort_by));
        }
        if sort_dir == SortDir::Desc {
            parts.push("sort_dir=desc".to_string());
        }
        if let Some(ref s) = search {
            parts.push(format!("search={}", urlencoding::encode(s)));
        }
        for (col, val) in &column_filters {
            parts.push(format!("filter_{}={}", col, urlencoding::encode(val)));
        }
        parts
    };
    let filter_meta: HashMap<String, serde_json::Value> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, col_limit)| {
            let cur_page = filter_pages.get(col).copied().unwrap_or(0);
            let total_distinct = filter_totals.get(col).copied().unwrap_or(0);
            let total_pages = total_distinct.div_ceil(*col_limit);
            let total_pages = total_pages.max(1);
            let has_prev = cur_page > 0;
            let has_next = cur_page.saturating_add(1) < total_pages;

            let build_qs = |fp_override: Option<u64>| -> String {
                let mut parts = base_qs.clone();
                for (other_col, other_page) in &filter_pages {
                    if other_col != col && *other_page > 0 {
                        parts.push(format!("fp_{}={}", other_col, other_page));
                    }
                }
                if let Some(fp) = fp_override
                    && fp > 0
                {
                    parts.push(format!("fp_{}={}", col, fp));
                }
                parts.join("&")
            };

            let prev_qs = if has_prev {
                build_qs(Some(cur_page.saturating_sub(1)))
            } else {
                String::new()
            };
            let next_qs = if has_next {
                build_qs(Some(cur_page.saturating_add(1)))
            } else {
                String::new()
            };

            let meta = serde_json::json!({
                "current_page": cur_page,
                "total_pages": total_pages,
                "has_prev": has_prev,
                "has_next": has_next,
                "prev_qs": prev_qs,
                "next_qs": next_qs,
            });
            (col.clone(), meta)
        })
        .collect();

    macro_rules! ctx {
        ($($key:expr => $val:expr),* $(,)?) => {
            $( req.context.insert($key, &$val); )*
        };
    }

    ctx! {
        list_ctx::LANG              => current_lang().code(),
        list_ctx::ENTRIES           => entries,
        list_ctx::TOTAL             => total,
        list_ctx::PAGE              => page,
        list_ctx::PAGE_COUNT        => page_count,
        list_ctx::HAS_PREV          => (page > 1),
        list_ctx::HAS_NEXT          => (page < page_count),
        list_ctx::PREV_PAGE         => page.saturating_sub(1),
        list_ctx::NEXT_PAGE         => page.saturating_add(1),
        "current_page"              => "list",
        list_ctx::VISIBLE_COLUMNS   => visible_columns,
        list_ctx::COLUMN_LABELS     => column_labels,
        list_ctx::SORT_BY           => safe_sort_by,
        list_ctx::SORT_DIR          => sort_dir.as_str(),
        list_ctx::SORT_DIR_TOGGLE   => sort_dir.toggle(),
        list_ctx::SEARCH            => search.unwrap_or_default(),
        list_ctx::FILTER_VALUES     => filter_values,
        list_ctx::ACTIVE_FILTERS    => active_filters,
        list_ctx::FILTER_QS         => filter_qs,
        list_ctx::FILTER_META       => filter_meta,
    }

    let htmx_tpl = state.config.templates.htmx.resolve().to_string();
    let template = if is_htmx {
        htmx_tpl.as_str()
    } else {
        entry
            .meta
            .template_list
            .as_deref()
            .unwrap_or_else(|| state.config.templates.list.resolve())
    };
    req.render(template)
}

async fn handle_create_get(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let tera: Arc<tera::Tera> = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        StrMap::new(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;

    req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_create::IS_EDIT, &false);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    let template = entry
        .meta
        .template_create
        .as_deref()
        .unwrap_or_else(|| state.config.templates.create.resolve());
    req.render(template)
}

async fn handle_create_post(
    req: &mut Request,
    entry: &ResourceEntry,
    mut body: StrMap,
    headers: &axum::http::HeaderMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    // If the resource declares inject_password (via create_form: in admin!{}),
    // inject a random hash into the empty "password" field.
    if entry.meta.inject_password && body.get("password").is_some_and(|p| p.is_empty()) {
        let temp_pw = uuid::Uuid::new_v4().to_string();
        if let Ok(hash) = crate::utils::password::hash(&temp_pw) {
            body.insert("password".to_string(), hash);
        }
    }

    let body_for_create = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let mut form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        body,
        tera,
        csrf,
        axum::http::Method::POST,
    )
    .await;

    if form.is_valid().await {
        let result = match &entry.create_fn {
            Some(f) => f(req.engine.db.clone(), body_for_create.clone()).await,
            None => form.save(&req.engine.db).await,
        };
        match result {
            Ok(()) => {}
            Err(sea_orm::DbErr::Custom(ref msg)) => {
                form.get_form_mut().errors.push(msg.clone());
                req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
                req.context.insert(ctx_create::IS_EDIT, &false);
                req.context.insert(ctx_common::LANG, &current_lang().code());
                let template = entry
                    .meta
                    .template_create
                    .as_deref()
                    .unwrap_or_else(|| state.config.templates.create.resolve());
                return req.render(template);
            }
            Err(e) => {
                form.get_form_mut().database_error(&e);
                if !is_unique_violation(&e) {
                    return Err(Box::new(AppError::new(ErrorContext::database(e))));
                }
                // uniqueness violation: fall through to form re-rendering
                req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
                req.context.insert(ctx_create::IS_EDIT, &false);
                req.context.insert(ctx_common::LANG, &current_lang().code());
                let template = entry
                    .meta
                    .template_create
                    .as_deref()
                    .unwrap_or_else(|| state.config.templates.create.resolve());
                return req.render(template);
            }
        }

        // Send welcome email + reset after creating an admin user
        if entry.meta.inject_password
            && let Some(email) = body_for_create.get("email")
        {
            let email_template = state
                .config
                .user_resources
                .get(entry.meta.key)
                .and_then(|t| t.as_deref());
            send_user_created_email(
                req,
                entry,
                email,
                body_for_create.get("username").map(String::as_str),
                email_template,
                headers,
                state,
            )
            .await;
        }

        req.notices
            .success(t("admin.create.success").to_string())
            .await;
        return Ok(Redirect::to(&format!(
            "{}/{}/list",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key
        ))
        .into_response());
    }

    req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_create::IS_EDIT, &false);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    let template = entry
        .meta
        .template_create
        .as_deref()
        .unwrap_or_else(|| state.config.templates.create.resolve());
    req.render(template)
}

async fn handle_detail(
    req: &mut Request,
    entry: &crate::admin::helper::ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(v) = &object {
        req.context.insert(ctx_detail::ENTRY, v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
    let template = entry
        .meta
        .template_detail
        .as_deref()
        .unwrap_or_else(|| state.config.templates.detail.resolve());
    req.render(template)
}

async fn handle_edit_get(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();

    // Pre-filling via get_fn if available
    let data = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?
            .map(value_to_strmap)
            .unwrap_or_default(),
        None => StrMap::new(),
    };

    let builder = entry
        .edit_form_builder
        .as_ref()
        .unwrap_or(&entry.form_builder);
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let form = (builder)(
        req.engine.db.clone(),
        resource_keys,
        data.clone(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;

    if let Some(ts) = data.get("updated_at") {
        req.context.insert("orig_updated_at", ts);
    }

    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

async fn handle_edit_post(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    body: StrMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let mut body_for_update = body.clone();
    let orig_updated_at = body_for_update.remove("__original_updated_at");

    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let builder = entry
        .edit_form_builder
        .as_ref()
        .unwrap_or(&entry.form_builder);
    // Method::PATCH signals edit mode — password fields relax their required constraint
    // (empty password = keep existing, handled by NotSet in admin_from_form)
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let mut form = (builder)(
        req.engine.db.clone(),
        resource_keys,
        body,
        tera,
        csrf,
        axum::http::Method::PATCH,
    )
    .await;

    let mut is_locked = false;

    if form.is_valid().await
        && let Some(orig_ts) = &orig_updated_at
        && let Some(get_fn) = &entry.get_fn
        && let Ok(Some(current_obj)) = get_fn(req.engine.db.clone(), id.clone()).await
        && let Some(current_ts) = current_obj.get("updated_at").and_then(|v| v.as_str())
        && current_ts != orig_ts
    {
        is_locked = true;
        form.get_form_mut().errors.push("Update failed: This content has been modified by another person during your editing. Please copy your changes and reload the page.".to_string());
        req.notices.error("This content has been modified by someone else during your editing. Refresh the page.").await;
    }

    if !is_locked && !form.get_form().has_errors() {
        let result = match &entry.update_fn {
            Some(f) => f(req.engine.db.clone(), id.clone(), body_for_update).await,
            None => form.save(&req.engine.db).await,
        };
        if let Err(e) = result {
            form.get_form_mut().database_error(&e);
            if !is_unique_violation(&e) {
                return Err(Box::new(AppError::new(ErrorContext::database(e))));
            }
            // uniqueness violation: fall through to form re-rendering
        } else {
            req.notices
                .success(t("admin.edit.success").to_string())
                .await;
            return Ok(Redirect::to(&format!(
                "{}/{}/list",
                state.config.prefix.trim_end_matches('/'),
                entry.meta.key
            ))
            .into_response());
        }
    }

    if let Some(ts) = orig_updated_at {
        req.context.insert("orig_updated_at", &ts);
    }

    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

async fn handle_delete_get(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(v) = &object {
        req.context.insert(ctx_detail::ENTRY, v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
    let template = entry
        .meta
        .template_delete
        .as_deref()
        .unwrap_or_else(|| state.config.templates.delete.resolve());
    req.render(template)
}

async fn handle_delete_post(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    delete_fn(req.engine.db.clone(), id)
        .await
        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    req.notices
        .success(t("admin.delete.success").to_string())
        .await;
    Ok(Redirect::to(&format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        entry.meta.key
    ))
    .into_response())
}

// ─── User creation — send reset email ────────────────────────────────

async fn send_user_created_email(
    req: &mut Request,
    _entry: &ResourceEntry,
    email: &str,
    username: Option<&str>,
    email_template: Option<&str>,
    headers: &axum::http::HeaderMap,
    state: &PrototypeAdminState,
) {
    let token = crate::utils::reset_token::generate(email);
    let encrypted = crate::utils::reset_token::encrypt_email(&token, email);

    let reset_url = if let Some(base) = &state.config.reset_password_url {
        format!("{}/{}/{}", base.trim_end_matches('/'), token, encrypted)
    } else {
        let host = headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        format!("http://{}/reset-password/{}/{}", host, token, encrypted)
    };

    let template_name = email_template.unwrap_or("admin/user_created_email.html");
    let username_str = username.unwrap_or(email);

    let mut ctx = tera::Context::new();
    ctx.insert("username", username_str);
    ctx.insert("email", email);
    ctx.insert("reset_url", &reset_url);
    ctx.insert(
        "t_greeting",
        t("admin.user_created.email_greeting").as_ref(),
    );
    ctx.insert("t_body", t("admin.user_created.email_body").as_ref());
    ctx.insert("t_btn", t("admin.user_created.email_btn").as_ref());
    ctx.insert(
        "t_validity",
        t("admin.user_created.email_validity").as_ref(),
    );

    let body_html = match req.engine.tera.render(template_name, &ctx) {
        Ok(rendered) => rendered,
        Err(_) => format!(
            "<p>Hello {username_str},</p><p>Click on the link to set your password:</p><p><a href=\"{reset_url}\">{reset_url}</a></p>"
        ),
    };

    if crate::utils::mailer_configured() {
        match crate::utils::Email::new()
            .to(email)
            .subject(t("admin.user_created.email_subject"))
            .html(body_html)
            .send()
            .await
        {
            Ok(_) => {
                req.notices
                    .success(crate::utils::trad::tf(
                        "admin.user_created.email_sent",
                        &[email],
                    ))
                    .await;
            }
            Err(e) => {
                req.notices
                    .error(crate::utils::trad::tf(
                        "admin.reset_password.error_send",
                        &[&e],
                    ))
                    .await;
            }
        }
    } else {
        req.notices
            .success(crate::utils::trad::tf(
                "admin.reset_password.success_link",
                &[&reset_url],
            ))
            .await;
    }
}

// ─── Reset password ──────────────────────────────────────────────────────────

async fn handle_reset_password(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    headers: &axum::http::HeaderMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    // Retrieve the entry to extract the email
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    let detail_url = format!(
        "{}/{}/{}/detail",
        state.config.prefix.trim_end_matches('/'),
        entry.meta.key,
        id
    );

    let fields = object.as_ref().and_then(|v| v.as_object());

    let email = fields
        .and_then(|m| m.get("email"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let Some(email) = email else {
        req.notices
            .error(t("admin.reset_password.error_no_email"))
            .await;
        return Ok(Redirect::to(&detail_url).into_response());
    };

    let username = fields
        .and_then(|m| m.get("username").or_else(|| m.get("name")))
        .and_then(|v| v.as_str())
        .unwrap_or(&email);

    let token = crate::utils::reset_token::generate(&email);
    let encrypted_email = crate::utils::reset_token::encrypt_email(&token, &email);

    let reset_url = if let Some(base) = &state.config.reset_password_url {
        format!(
            "{}/{}/{}",
            base.trim_end_matches('/'),
            token,
            encrypted_email
        )
    } else {
        let host = headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        format!(
            "http://{}/reset-password/{}/{}",
            host, token, encrypted_email
        )
    };

    // Send by email if mailer is configured, otherwise display the link in the flash message
    if crate::utils::mailer_configured() {
        let template_name = state
            .config
            .reset_password_email_template
            .as_deref()
            .unwrap_or("admin/reset_password_email.html");
        let mut ctx = tera::Context::new();
        ctx.insert("username", username);
        ctx.insert("email", &email);
        ctx.insert("reset_url", &reset_url);
        ctx.insert("t_title", t("admin.reset_password.email_title").as_ref());
        ctx.insert(
            "t_greeting",
            t("admin.reset_password.email_greeting").as_ref(),
        );
        ctx.insert("t_body", t("admin.reset_password.email_body").as_ref());
        ctx.insert("t_btn", t("admin.reset_password.btn").as_ref());
        ctx.insert("t_ignore", t("admin.reset_password.email_ignore").as_ref());
        let body = match req.engine.tera.render(template_name, &ctx) {
            Ok(rendered) => rendered,
            Err(_) => format!(
                "<p>Hello {username},</p><p>Click on the following link to reset your password (valid for 1 hour):</p><p><a href=\"{reset_url}\">{reset_url}</a></p>"
            ),
        };
        match crate::utils::Email::new()
            .to(email.clone())
            .subject(t("admin.reset_password.email_subject"))
            .html(body)
            .send()
            .await
        {
            Ok(_) => {
                req.notices
                    .success(crate::utils::trad::tf(
                        "admin.reset_password.success_email",
                        &[&email],
                    ))
                    .await;
            }
            Err(e) => {
                req.notices
                    .error(crate::utils::trad::tf(
                        "admin.reset_password.error_send",
                        &[&e],
                    ))
                    .await;
            }
        }
    } else {
        req.notices
            .success(crate::utils::trad::tf(
                "admin.reset_password.success_link",
                &[&reset_url],
            ))
            .await;
    }

    Ok(Redirect::to(&detail_url).into_response())
}
