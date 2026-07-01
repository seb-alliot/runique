//! Generic CRUD handler for the admin interface.
//!
//! Covered routes:
//! - `GET/POST /admin/{resource}/{action}` → [`admin_get`] / [`admin_post`]
//! - `GET/POST /admin/{resource}/{id}/{action}` → [`admin_get_id`] / [`admin_post_id`]

mod action;
mod handle_bulk;
mod handle_crud;
mod handle_inline;
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

// ─── Route target ────────────────────────────────────────────

/// The routing identity of a request, built by the thin flat/nested wrappers and
/// passed to the shared dispatchers — keeps their signatures small and lets flat
/// and nested share one code path.
struct RouteTarget {
    resource_key: String,
    action: String,
    /// Raw `(parent_key, parent_id)` from a nested URL; `None` for a flat request.
    parent: Option<(String, String)>,
}

impl RouteTarget {
    fn flat(resource_key: String, action: String) -> Self {
        Self {
            resource_key,
            action,
            parent: None,
        }
    }

    fn nested(parent_key: String, parent_id: String, resource_key: String, action: String) -> Self {
        Self {
            resource_key,
            action,
            parent: Some((parent_key, parent_id)),
        }
    }
}

// ─── Parent scope binding (nested resources) ─────────────────

/// A resolved parent scope for a nested-resource request
/// (`/{parent_key}/{parent_id}/{child}/...`). Built and validated by the nested
/// entry points from the URL against the child's declared [`ParentScope`], then
/// threaded through the shared CRUD logic so flat and nested requests share a
/// single code path.
#[derive(Clone)]
pub(super) struct ParentBinding {
    pub parent_key: String,
    pub parent_id: String,
    /// Child FK column holding the parent id (from the child's `ParentScope`).
    pub fk_col: &'static str,
    /// `Some(col)` for a composite/junction child keyed by `(fk_col, col)`;
    /// `None` when the child owns its own primary key.
    pub local_key: Option<&'static str>,
}

impl ParentBinding {
    /// Whether the child's closure-id is composite `"{parent_id}:{local}"`.
    pub(super) fn is_composite(&self) -> bool {
        self.local_key.is_some()
    }

    /// Turns a local URL id segment into the id the CRUD closures expect.
    /// Composite children get the parent prefix rebuilt; others pass through.
    pub(super) fn closure_id(&self, local: &str) -> String {
        if self.is_composite() {
            format!("{}:{}", self.parent_id, local)
        } else {
            local.to_string()
        }
    }

    /// Strips the composite parent prefix from a closure id, yielding the local
    /// segment used in URLs. Non-composite ids pass through unchanged.
    pub(super) fn local_id<'a>(&self, closure_id: &'a str) -> &'a str {
        if self.is_composite() {
            closure_id
                .split_once(':')
                .map_or(closure_id, |(_, local)| local)
        } else {
            closure_id
        }
    }

    /// Scope-aware base path for building action URLs in templates.
    pub(super) fn base_path(&self, prefix: &str, child_key: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            prefix.trim_end_matches('/'),
            self.parent_key,
            self.parent_id,
            child_key
        )
    }
}

/// Base path for a flat (non-nested) resource.
fn flat_base_path(prefix: &str, key: &str) -> String {
    format!("{}/{}", prefix.trim_end_matches('/'), key)
}

/// Resolves and validates a nested request's parent binding.
///
/// Returns `None` (→ caller answers 404) when the child is not declared as a
/// scoped child of that parent — this is the guard that forbids reaching an
/// unrelated resource through an arbitrary parent (`/groupes/2/user/...`).
fn build_parent_binding(
    meta: &crate::admin::resource::AdminResource,
    parent_key: &str,
    parent_id: &str,
) -> Option<ParentBinding> {
    let scope = meta.parent_scope.as_ref()?;
    if scope.parent_key != parent_key {
        return None;
    }
    Some(ParentBinding {
        parent_key: parent_key.to_string(),
        parent_id: parent_id.to_string(),
        fk_col: scope.fk_col,
        local_key: scope.local_key,
    })
}

/// Resolves the parent binding for a nested request, validating it against the
/// child's declared `ParentScope` (else 404 — forbids reaching an unrelated
/// resource through an arbitrary parent, `/{unrelated_parent}/{id}/{child}/...`).
///
/// A flat request is always allowed (`Ok(None)`), including on a scoped child:
/// the child is merely hidden from the nav (see `AdminRegistry::visible_to`), not
/// route-blocked, so a superuser keeps direct access as a bypass and the flat
/// route stays reachable by URL for anyone with the resource permission.
fn resolve_scope(
    meta: &crate::admin::resource::AdminResource,
    parent: Option<(String, String)>,
) -> AppResult<Option<ParentBinding>> {
    match parent {
        Some((pk, pid)) => build_parent_binding(meta, &pk, &pid)
            .map(Some)
            .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found")))),
        None => Ok(None),
    }
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

/// GET /admin/{resource}/{action}  (list, create) — flat (top-level).
pub async fn admin_get(
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    headers: axum::http::HeaderMap,
    req: Request,
) -> AppResult<Response> {
    dispatch_collection_get(
        RouteTarget::flat(resource_key, action),
        state,
        current_user,
        params,
        headers,
        req,
    )
    .await
}

/// GET /admin/{parent}/{parent_id}/{resource}/{action} — nested (scoped child).
pub async fn admin_nested_get(
    Path((parent_key, parent_id, resource_key, action)): Path<(String, String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    headers: axum::http::HeaderMap,
    req: Request,
) -> AppResult<Response> {
    dispatch_collection_get(
        RouteTarget::nested(parent_key, parent_id, resource_key, action),
        state,
        current_user,
        params,
        headers,
        req,
    )
    .await
}

async fn dispatch_collection_get(
    target: RouteTarget,
    state: Arc<PrototypeAdminState>,
    current_user: CurrentUser,
    params: StrMap,
    headers: axum::http::HeaderMap,
    mut req: Request,
) -> AppResult<Response> {
    let RouteTarget {
        resource_key,
        action,
        parent,
    } = target;
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let parent = resolve_scope(&entry.meta, parent)?;
    let perms = inject_context(&mut req, &state, entry, &current_user, parent.as_ref());

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
    let base = scope_base(&state.config.prefix, entry, parent.as_ref());
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &base).await {
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
                scope: parent
                    .as_ref()
                    .map(|p| (p.fk_col.to_string(), p.parent_id.clone())),
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
            handle_list(
                &mut req,
                entry,
                &state,
                query,
                &current_user,
                is_htmx,
                parent.as_ref(),
            )
            .await
        }
        CollectionAction::Create => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, action = "create GET", "crud");
            }
            handle_create_get(&mut req, entry, &state, parent.as_ref()).await
        }
        CollectionAction::Bulk => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.bulk)
            {
                crate::runique_log!(level, resource = %resource_key, action = "bulk GET", "bulk");
            }
            handle_bulk::handle_bulk_edit_get(&mut req, entry, &state, &params, parent.as_ref())
                .await
        }
    }
}

/// POST /admin/{resource}/{action}  (create, bulk) — flat.
#[allow(private_interfaces)]
pub async fn admin_post(
    headers: axum::http::HeaderMap,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_collection_post(
        RouteTarget::flat(resource_key, action),
        state,
        current_user,
        headers,
        req,
    )
    .await
}

/// POST /admin/{parent}/{parent_id}/{resource}/{action} — nested.
#[allow(private_interfaces)]
pub async fn admin_nested_post(
    headers: axum::http::HeaderMap,
    Path((parent_key, parent_id, resource_key, action)): Path<(String, String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_collection_post(
        RouteTarget::nested(parent_key, parent_id, resource_key, action),
        state,
        current_user,
        headers,
        req,
    )
    .await
}

async fn dispatch_collection_post(
    target: RouteTarget,
    state: Arc<PrototypeAdminState>,
    current_user: CurrentUser,
    headers: axum::http::HeaderMap,
    mut req: Request,
) -> AppResult<Response> {
    let RouteTarget {
        resource_key,
        action,
        parent,
    } = target;
    let body = req.prisme.data.clone();
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let parent = resolve_scope(&entry.meta, parent)?;
    let perms = inject_context(&mut req, &state, entry, &current_user, parent.as_ref());
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
    let base = scope_base(&state.config.prefix, entry, parent.as_ref());
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &base).await {
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
            handle_create_post(
                &mut req,
                entry,
                body,
                &headers,
                &state,
                &current_user,
                parent.as_ref(),
            )
            .await
        }
        CollectionAction::Bulk => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.bulk)
            {
                crate::runique_log!(level, resource = %resource_key, action = "bulk POST", "bulk");
            }
            handle_bulk_action(
                &mut req,
                entry,
                body,
                &state,
                &current_user,
                parent.as_ref(),
            )
            .await
        }
        // `list` is rejected by `parse_post`; unreachable.
        CollectionAction::List => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// GET /admin/{resource}/{id}/{action}  (detail, edit, delete) — flat.
pub async fn admin_get_id(
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_member_get(
        RouteTarget::flat(resource_key, action),
        id,
        state,
        current_user,
        req,
    )
    .await
}

/// GET /admin/{parent}/{parent_id}/{resource}/{id}/{action} — nested.
pub async fn admin_nested_get_id(
    Path((parent_key, parent_id, resource_key, id, action)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_member_get(
        RouteTarget::nested(parent_key, parent_id, resource_key, action),
        id,
        state,
        current_user,
        req,
    )
    .await
}

async fn dispatch_member_get(
    target: RouteTarget,
    id: String,
    state: Arc<PrototypeAdminState>,
    current_user: CurrentUser,
    mut req: Request,
) -> AppResult<Response> {
    let RouteTarget {
        resource_key,
        action,
        parent,
    } = target;
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let parent = resolve_scope(&entry.meta, parent)?;
    let perms = inject_context(&mut req, &state, entry, &current_user, parent.as_ref());
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
    let closure_id = closure_id_of(parent.as_ref(), &id);
    if let Some(p) = parent.as_ref()
        && !verify_scope_ownership(entry, req.engine.db.clone(), &closure_id, p).await
    {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Resource not found",
        ))));
    }
    let owns_record =
        check_owns_record(entry, req.engine.db.clone(), &closure_id, current_user.id).await;
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
    let base = scope_base(&state.config.prefix, entry, parent.as_ref());
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &base).await {
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
            handle_detail(&mut req, entry, id, &state, parent.as_ref(), &current_user).await
        }
        MemberAction::Edit => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "edit GET", "crud");
            }
            handle_edit_get(&mut req, entry, id, &state, parent.as_ref()).await
        }
        MemberAction::Delete => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "delete GET", "crud");
            }
            handle_delete_get(&mut req, entry, id, &state, parent.as_ref()).await
        }
        // `reset-password` is POST-only; rejected by `parse_get`, unreachable.
        MemberAction::ResetPassword => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        )))),
    }
}

/// POST /admin/{resource}/{id}/{action}  (edit, delete, reset-password) — flat.
#[allow(private_interfaces)]
pub async fn admin_post_id(
    headers: axum::http::HeaderMap,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_member_post(
        RouteTarget::flat(resource_key, action),
        id,
        state,
        current_user,
        headers,
        req,
    )
    .await
}

/// POST /admin/{parent}/{parent_id}/{resource}/{id}/{action} — nested.
#[allow(private_interfaces)]
pub async fn admin_nested_post_id(
    headers: axum::http::HeaderMap,
    Path((parent_key, parent_id, resource_key, id, action)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    dispatch_member_post(
        RouteTarget::nested(parent_key, parent_id, resource_key, action),
        id,
        state,
        current_user,
        headers,
        req,
    )
    .await
}

async fn dispatch_member_post(
    target: RouteTarget,
    id: String,
    state: Arc<PrototypeAdminState>,
    current_user: CurrentUser,
    headers: axum::http::HeaderMap,
    mut req: Request,
) -> AppResult<Response> {
    let RouteTarget {
        resource_key,
        action,
        parent,
    } = target;
    let body = req.prisme.data.clone();
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    let parent = resolve_scope(&entry.meta, parent)?;
    let perms = inject_context(&mut req, &state, entry, &current_user, parent.as_ref());
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;

    let Some(act) = MemberAction::parse_post(&action) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown action",
        ))));
    };
    let closure_id = closure_id_of(parent.as_ref(), &id);
    if let Some(p) = parent.as_ref()
        && !verify_scope_ownership(entry, req.engine.db.clone(), &closure_id, p).await
    {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Resource not found",
        ))));
    }
    let owns_record =
        check_owns_record(entry, req.engine.db.clone(), &closure_id, current_user.id).await;
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
    let base = scope_base(&state.config.prefix, entry, parent.as_ref());
    if let Some(resp) = enforce(access, &req.notices, &state.config.prefix, &base).await {
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
            handle_edit_post(
                &mut req,
                entry,
                id,
                body,
                &state,
                &current_user,
                parent.as_ref(),
            )
            .await
        }
        MemberAction::Delete => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %resource_key, id = %id, action = "delete POST", "crud");
            }
            handle_delete_post(&mut req, entry, id, &state, &current_user, parent.as_ref()).await
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
    parent: Option<&ParentBinding>,
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

    // Single source of truth for action URLs: templates build every link from
    // `resource_base`, which is scope-aware (flat or nested). See Bloc 6.
    req.context.insert(
        "resource_base",
        &scope_base(&state.config.prefix, entry, parent),
    );
    match parent {
        Some(p) => {
            req.context.insert("parent_key", &p.parent_key);
            req.context.insert("parent_id", &p.parent_id);
            let parent_title = state
                .registry
                .get(&p.parent_key)
                .map(|e| e.meta.title)
                .unwrap_or(p.parent_key.as_str());
            req.context.insert("parent_title", &parent_title);
            req.context.insert(
                "parent_base",
                &flat_base_path(&state.config.prefix, &p.parent_key),
            );
        }
        None => {
            req.context.insert("parent_key", &Option::<String>::None);
        }
    }

    let visible_resources = state.registry.visible_to(current_user);
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

/// Scope-aware base path (`{prefix}/{key}` flat, `{prefix}/{parent}/{id}/{key}`
/// nested) — the single place action URLs are assembled server-side.
pub(super) fn scope_base(
    prefix: &str,
    entry: &ResourceEntry,
    parent: Option<&ParentBinding>,
) -> String {
    match parent {
        Some(p) => p.base_path(prefix, entry.meta.key),
        None => flat_base_path(prefix, entry.meta.key),
    }
}

/// Maps an [`Access`] decision to a redirect response, or `None` when granted.
/// `base` is the scope-aware resource base (`enforce` redirects to `{base}/list`
/// on a resource-level denial) so a nested denial never bounces to the blocked
/// flat route.
async fn enforce(
    access: Access,
    notices: &crate::flash::flash_manager::Message,
    prefix: &str,
    base: &str,
) -> Option<Response> {
    match access {
        Access::Granted => None,
        Access::DeniedDashboard => Some(permission_denied_dashboard(notices, prefix).await),
        Access::DeniedResource => Some(permission_denied(notices, base).await),
    }
}

/// Turns a local URL id segment into the id the CRUD closures expect
/// (rebuilds the composite parent prefix for nested composite children).
pub(super) fn closure_id_of(parent: Option<&ParentBinding>, local: &str) -> String {
    parent.map_or_else(|| local.to_string(), |p| p.closure_id(local))
}

/// IDOR guard for a scoped child with its **own** PK (non-composite): verifies
/// the target row actually belongs to the bound parent (`row[fk_col] == parent_id`),
/// forbidding `/{parent}/A/{child}/{id}` from reaching a child of parent B.
/// Composite children need no check — the parent id is baked into the closure id.
async fn verify_scope_ownership(
    entry: &ResourceEntry,
    db: crate::utils::aliases::ADb,
    closure_id: &str,
    parent: &ParentBinding,
) -> bool {
    if parent.is_composite() {
        return true;
    }
    let Some(get_fn) = &entry.get_fn else {
        return false;
    };
    match get_fn(db, closure_id.to_string()).await {
        Ok(Some(row)) => row
            .get(parent.fk_col)
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .is_some_and(|v| v == parent.parent_id),
        Ok(None) => false,
        Err(e) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.auth)
            {
                crate::runique_log!(
                    level,
                    resource = entry.meta.key,
                    id = %closure_id,
                    error = %e,
                    "scope ownership check failed — denying access"
                );
            }
            false
        }
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

async fn permission_denied(notices: &crate::flash::flash_manager::Message, base: &str) -> Response {
    notices
        .error(t("admin.access.insufficient_rights").to_string())
        .await;
    Redirect::to(&format!("{}/list", base)).into_response()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::resource::AdminResource;

    fn meta_child() -> AdminResource {
        AdminResource::new("droits", "M", "F", "Droits", vec![]).parent_scope(
            "groupes",
            "groupe_id",
            Some("resource_key"),
        )
    }

    fn meta_own_pk_child() -> AdminResource {
        AdminResource::new("lignes", "M", "F", "Lignes", vec![]).parent_scope(
            "commandes",
            "commande_id",
            None,
        )
    }

    fn meta_flat() -> AdminResource {
        AdminResource::new("menus", "M", "F", "Menus", vec![])
    }

    fn binding_composite() -> ParentBinding {
        ParentBinding {
            parent_key: "groupes".into(),
            parent_id: "2".into(),
            fk_col: "groupe_id",
            local_key: Some("resource_key"),
        }
    }

    fn binding_own_pk() -> ParentBinding {
        ParentBinding {
            parent_key: "commandes".into(),
            parent_id: "5".into(),
            fk_col: "commande_id",
            local_key: None,
        }
    }

    #[test]
    fn composite_closure_id_rebuilds_prefix() {
        let b = binding_composite();
        assert!(b.is_composite());
        assert_eq!(b.closure_id("changelog_entry"), "2:changelog_entry");
    }

    #[test]
    fn composite_local_id_strips_prefix() {
        let b = binding_composite();
        assert_eq!(b.local_id("2:changelog_entry"), "changelog_entry");
        // A colon inside the local key must survive (only the first split counts).
        assert_eq!(b.local_id("2:a:b"), "a:b");
    }

    #[test]
    fn own_pk_child_ids_pass_through() {
        let b = binding_own_pk();
        assert!(!b.is_composite());
        assert_eq!(b.closure_id("42"), "42");
        assert_eq!(b.local_id("42"), "42");
    }

    #[test]
    fn base_paths_are_scope_aware() {
        let b = binding_composite();
        assert_eq!(b.base_path("/admin", "droits"), "/admin/groupes/2/droits");
        assert_eq!(b.base_path("/admin/", "droits"), "/admin/groupes/2/droits");
        assert_eq!(flat_base_path("/admin", "menus"), "/admin/menus");
    }

    #[test]
    fn build_binding_requires_matching_parent() {
        // Correct parent → binding built.
        let b = build_parent_binding(&meta_child(), "groupes", "2").unwrap();
        assert_eq!(b.fk_col, "groupe_id");
        assert_eq!(b.local_key, Some("resource_key"));
        // Wrong parent key → rejected (forbids /wrong_parent/2/droits/...).
        assert!(build_parent_binding(&meta_child(), "menus", "2").is_none());
        // Resource without a parent_scope → never a valid child.
        assert!(build_parent_binding(&meta_flat(), "groupes", "2").is_none());
    }

    #[test]
    fn resolve_scope_nested_valid() {
        let r = resolve_scope(&meta_child(), Some(("groupes".into(), "2".into())));
        let binding = r.ok().flatten().expect("some binding");
        assert_eq!(binding.parent_id, "2");
    }

    #[test]
    fn resolve_scope_nested_wrong_parent_is_404() {
        let r = resolve_scope(&meta_child(), Some(("menus".into(), "2".into())));
        assert!(r.is_err());
    }

    #[test]
    fn resolve_scope_flat_on_scoped_child_allowed() {
        // A scoped child is only hidden from the nav, not route-blocked: a flat
        // request stays allowed (superuser bypass / direct URL access).
        let r = resolve_scope(&meta_child(), None);
        assert!(matches!(r, Ok(None)));
    }

    #[test]
    fn resolve_scope_flat_normal_resource_ok() {
        let r = resolve_scope(&meta_flat(), None);
        assert!(matches!(r, Ok(None)));
    }

    #[test]
    fn resolve_scope_own_pk_child_nested_ok() {
        let r = resolve_scope(&meta_own_pk_child(), Some(("commandes".into(), "5".into())));
        let binding = r.ok().flatten().expect("some");
        assert!(!binding.is_composite());
        assert_eq!(binding.fk_col, "commande_id");
    }
}
