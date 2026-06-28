//! Generic CRUD handler for the admin interface.
//!
//! Covered routes:
//! - `GET/POST /admin/{resource}/{action}` → [`admin_get`] / [`admin_post`]
//! - `GET/POST /admin/{resource}/{id}/{action}` → [`admin_get_id`] / [`admin_post_id`]

mod action;
mod handle_bulk;
mod handle_crud;
mod handle_list;
mod handle_password;

use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::config::TraceResult;
use crate::utils::{
    aliases::{AppResult, StrMap},
    constante::admin_context::{common as ctx_common, list as ctx_list, permission as ctx_perm},
    session_key::session::CSRF_TOKEN_KEY,
    trad::{current_lang, t},
};
use crate::{
    admin::{
        AdminRegistry,
        config::AdminConfig,
        helper::resource_entry::{ResourceEntry, SortDir},
        trad::{inject_admin_prefix, insert_admin_messages},
    },
    utils::admin_context::list::{PAGE, SORT_BY, SORT_DIR},
};
use axum::{
    Extension,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use std::{collections::HashMap, sync::Arc};
use subtle::ConstantTimeEq;

use self::action::{Access, CollectionAction, MemberAction};
use self::handle_bulk::handle_bulk_action;
use self::handle_crud::{
    handle_create_get, handle_create_post, handle_delete_get, handle_delete_post, handle_detail,
    handle_edit_get, handle_edit_post,
};
use self::handle_list::{ListQuery, handle_list};
use self::handle_password::handle_reset_password;

// ─── Datetime formatting ─────────────────────────────────────

pub(crate) fn format_datetime(value: &mut serde_json::Value) {
    use chrono::NaiveDateTime;
    match value {
        serde_json::Value::String(s) => {
            if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f"))
                .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
            {
                *s = dt.format("%d/%m/%Y %H:%M").to_string();
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values_mut() {
                format_datetime(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                format_datetime(v);
            }
        }
        _ => {}
    }
}

// ─── Shared state ────────────────────────────────────────────

#[derive(Clone)]
pub struct PrototypeAdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
}

// ─── Resolved permissions ────────────────────────────────────

/// Effective CRUD rights of the current user on a single resource.
///
/// Single source of truth shared by the template context (`inject_context`)
/// and the server-side access checks (`admin_get_id`/`admin_post_id`), so the
/// rendered UI and the enforced policy can never diverge.
#[derive(Clone, Copy)]
pub(super) struct ResourcePerms {
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
}

impl ResourcePerms {
    pub(super) fn resolve(user: &CurrentUser, resource_key: &str) -> Self {
        if user.is_superuser {
            return Self {
                can_create: true,
                can_read: true,
                can_update: true,
                can_delete: true,
                can_update_own: true,
                can_delete_own: true,
            };
        }
        match user.permission_for(resource_key) {
            Some(p) => Self {
                can_create: p.can_create,
                can_read: p.can_read,
                can_update: p.can_update,
                can_delete: p.can_delete,
                can_update_own: p.can_update_own,
                can_delete_own: p.can_delete_own,
            },
            None => Self {
                can_create: false,
                can_read: false,
                can_update: false,
                can_delete: false,
                can_update_own: false,
                can_delete_own: false,
            },
        }
    }

    /// Edit allowed if global update, or own-update on a record the user owns.
    pub(super) fn can_edit(&self, owns_record: bool) -> bool {
        self.can_update || (self.can_update_own && owns_record)
    }

    /// Delete allowed if global delete, or own-delete on a record the user owns.
    pub(super) fn can_remove(&self, owns_record: bool) -> bool {
        self.can_delete || (self.can_delete_own && owns_record)
    }
}

// ─── Axum entry points ────────────────────────────────────

/// GET /admin/{resource}/{action}  (list, create)
pub async fn admin_get(
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    headers: axum::http::HeaderMap,
    mut req: Request,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let perms = inject_context(&mut req, &state, entry, &current_user);

    let Some(act) = CollectionAction::parse_get(&action) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        ))));
    };
    let access = act.authorize_get(&perms);
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        crate::runique_log!(
            level,
            resource = %resource_key,
            action = %action,
            user = %current_user.username,
            granted = matches!(access, Access::Granted),
            "collection GET access check"
        );
    }
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &resource_key).await {
        return Ok(resp);
    }

    match act {
        CollectionAction::List => {
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
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.list)
            {
                crate::runique_log!(
                    level,
                    resource = %resource_key,
                    page = query.page,
                    search = ?query.search,
                    filters = query.column_filters.len(),
                    htmx = is_htmx,
                    "list"
                );
            }
            handle_list(&mut req, entry, &state, query, &current_user, is_htmx).await
        }
        CollectionAction::Create => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, action = "create GET", "crud");
            }
            handle_create_get(&mut req, entry, &state).await
        }
        CollectionAction::Bulk => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.bulk)
            {
                crate::runique_log!(level, resource = %resource_key, action = "bulk GET", "bulk");
            }
            handle_bulk::handle_bulk_edit_get(&mut req, entry, &state, &params).await
        }
    }
}

/// POST /admin/{resource}/{action}  (create)
#[allow(private_interfaces)]
pub async fn admin_post(
    headers: axum::http::HeaderMap,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let body = req.prisme.data.clone();
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let perms = inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;

    let Some(act) = CollectionAction::parse_post(&action) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        ))));
    };
    let bulk_action = body.get("bulk_action").map(String::as_str).unwrap_or("");
    let access = act.authorize_post(&perms, bulk_action);
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        crate::runique_log!(
            level,
            resource = %resource_key,
            user = %current_user.username,
            action = %action,
            granted = matches!(access, Access::Granted),
            "collection POST access check"
        );
    }
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &resource_key).await {
        return Ok(resp);
    }

    match act {
        CollectionAction::Create => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, action = "create POST", "crud");
            }
            handle_create_post(&mut req, entry, body, &headers, &state, &current_user).await
        }
        CollectionAction::Bulk => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.bulk)
            {
                crate::runique_log!(level, resource = %resource_key, action = "bulk POST", "bulk");
            }
            handle_bulk_action(&mut req, entry, body, &state, &resource_key, &current_user).await
        }
        // `list` is rejected by `parse_post`; unreachable.
        CollectionAction::List => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// GET /admin/{resource}/{id}/{action}  (detail, edit, delete)
pub async fn admin_get_id(
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let perms = inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());

    let Some(act) = MemberAction::parse_get(&action) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        ))));
    };
    // Resource-level gate: must be able to see the resource at all.
    if !perms.can_read {
        return Ok(permission_denied_dashboard(&req.notices, &state.config.prefix).await);
    }
    let owns_record = check_owns_record(entry, req.engine.db.clone(), &id, current_user.id).await;
    let access = act.authorize(&perms, owns_record);
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        crate::runique_log!(
            level,
            resource = %resource_key,
            id = %id,
            action = %action,
            user = %current_user.username,
            granted = matches!(access, Access::Granted),
            "member GET access check"
        );
    }
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &resource_key).await {
        return Ok(resp);
    }
    match act {
        MemberAction::Detail => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "detail", "crud");
            }
            handle_detail(&mut req, entry, id, &state).await
        }
        MemberAction::Edit => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "edit GET", "crud");
            }
            handle_edit_get(&mut req, entry, id, &state).await
        }
        MemberAction::Delete => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "delete GET", "crud");
            }
            handle_delete_get(&mut req, entry, id, &state).await
        }
        // `reset-password` is POST-only; rejected by `parse_get`, unreachable.
        MemberAction::ResetPassword => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// POST /admin/{resource}/{id}/{action}  (edit, delete)
#[allow(private_interfaces)]
pub async fn admin_post_id(
    headers: axum::http::HeaderMap,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let body = req.prisme.data.clone();
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let perms = inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;

    let Some(act) = MemberAction::parse_post(&action) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        ))));
    };
    let owns_record = check_owns_record(entry, req.engine.db.clone(), &id, current_user.id).await;
    let access = act.authorize(&perms, owns_record);
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.auth)
    {
        crate::runique_log!(
            level,
            resource = %resource_key,
            id = %id,
            action = %action,
            user = %current_user.username,
            granted = matches!(access, Access::Granted),
            "member POST access check"
        );
    }
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &resource_key).await {
        return Ok(resp);
    }

    match act {
        MemberAction::Edit => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "edit POST", "crud");
            }
            handle_edit_post(&mut req, entry, id, body, &state, &current_user).await
        }
        MemberAction::Delete => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "delete POST", "crud");
            }
            handle_delete_post(&mut req, entry, id, &state, &current_user).await
        }
        MemberAction::ResetPassword => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "reset-password", "crud");
            }
            handle_reset_password(&mut req, entry, id, &headers, &state).await
        }
        // `detail` is GET-only; rejected by `parse_post`, unreachable.
        MemberAction::Detail => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

// ─── Shared helpers ──────────────────────────────────────────

pub(super) fn inject_context(
    req: &mut Request,
    state: &PrototypeAdminState,
    entry: &ResourceEntry,
    current_user: &CurrentUser,
) -> ResourcePerms {
    for item in ["list", "create", "edit", "detail", "delete", "base"] {
        insert_admin_messages(&mut req.context, item);
    }

    req.context
        .insert(ctx_common::SITE_TITLE, &state.config.site_title);
    req.context
        .insert(ctx_common::SITE_URL, &state.config.site_url);
    inject_admin_prefix(&mut req.context, &state.config.prefix);
    req.context.insert(ctx_common::RESOURCE_KEY, entry.meta.key);
    req.context
        .insert(ctx_common::CURRENT_RESOURCE, entry.meta.key);
    req.context.insert(ctx_common::RESOURCE, &entry.meta);
    req.context
        .insert(ctx_list::GROUP_ACTIONS, &entry.group_actions);

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

    let perms = ResourcePerms::resolve(current_user, entry.meta.key);
    req.context.insert(ctx_perm::CAN_CREATE, &perms.can_create);
    req.context.insert(ctx_perm::CAN_READ, &perms.can_read);
    req.context.insert(ctx_perm::CAN_UPDATE, &perms.can_update);
    req.context.insert(ctx_perm::CAN_DELETE, &perms.can_delete);
    req.context
        .insert(ctx_perm::CAN_UPDATE_OWN, &perms.can_update_own);
    req.context
        .insert(ctx_perm::CAN_DELETE_OWN, &perms.can_delete_own);

    perms
}

/// Maps an [`Access`] decision to a redirect response, or `None` when granted.
async fn enforce(
    access: Access,
    notices: &crate::flash::flash_manager::Message,
    prefix: &str,
    resource_key: &str,
) -> Option<Response> {
    match access {
        Access::Granted => None,
        Access::DeniedDashboard => Some(permission_denied_dashboard(notices, prefix).await),
        Access::DeniedResource => Some(permission_denied(notices, prefix, resource_key).await),
    }
}

/// Returns true if the record identified by `id` has `entry.own_field == user_id`.
/// Falls back to false when own_field is not set or get_fn is unavailable.
async fn check_owns_record(
    entry: &super::helper::resource_entry::ResourceEntry,
    db: crate::utils::aliases::ADb,
    id: &str,
    user_id: crate::utils::pk::Pk,
) -> bool {
    let own_field = match entry.own_field {
        Some(f) => f,
        None => return false,
    };
    let get_fn = match &entry.get_fn {
        Some(f) => f,
        None => return false,
    };
    let record = match get_fn(db, id.to_string())
        .await
        .trace(
            crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.auth),
            "load record for object permission check",
        )
        .flatten()
    {
        Some(v) => v,
        None => return false,
    };
    let field_val = match record.get(own_field) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => return false,
    };
    field_val == user_id.to_string()
}

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

pub(super) async fn permission_denied_dashboard(
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
        if let Some(level) = crate::utils::runique_log::get_log()
            .admin
            .as_ref()
            .and_then(|a| a.auth)
        {
            crate::runique_log!(level, "CSRF validation failed");
        }
        return Err(Box::new(AppError::new(ErrorContext::generic(
            StatusCode::FORBIDDEN,
            t("csrf.invalid_or_missing").as_ref(),
        ))));
    }
    Ok(())
}
