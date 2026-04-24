//! Generic CRUD handler for the admin interface.
//!
//! Covered routes:
//! - `GET/POST /admin/{resource}/{action}` → [`admin_get`] / [`admin_post`]
//! - `GET/POST /admin/{resource}/{id}/{action}` → [`admin_get_id`] / [`admin_post_id`]

mod handle_bulk;
mod handle_crud;
mod handle_list;
mod handle_password;

use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::{AppResult, StrMap},
    constante::admin_context::{common as ctx_common, permission as ctx_perm},
    session_key::session::CSRF_TOKEN_KEY,
    trad::{current_lang, t},
};
use crate::{
    admin::{
        AdminRegistry,
        config::AdminConfig,
        helper::resource_entry::{ResourceEntry, SortDir},
        trad::insert_admin_messages,
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

use self::handle_bulk::handle_bulk_action;
use self::handle_crud::{
    handle_create_get, handle_create_post, handle_delete_get, handle_delete_post, handle_detail,
    handle_edit_get, handle_edit_post,
};
use self::handle_list::{ListQuery, handle_list};
use self::handle_password::handle_reset_password;

// ─── Shared state ────────────────────────────────────────────

#[derive(Clone)]
pub struct PrototypeAdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
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
    headers: axum::http::HeaderMap,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    Extension(current_user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let body = req.prisme.data.clone();
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
        "create" => {
            handle_create_post(&mut req, entry, body, &headers, &state, &current_user).await
        }
        "bulk" => handle_bulk_action(&mut req, entry, body, &state, &resource_key).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
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

    inject_context(&mut req, &state, entry, &current_user);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;
    if !check_write_access(&current_user, &resource_key) {
        return Ok(permission_denied(&req.notices, &state.config.prefix, &resource_key).await);
    }

    match action.as_str() {
        "edit" => handle_edit_post(&mut req, entry, id, body, &state, &current_user).await,
        "delete" => handle_delete_post(&mut req, entry, id, &state, &current_user).await,
        "reset-password" => handle_reset_password(&mut req, entry, id, &headers, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
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

    let (can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) =
        if current_user.is_superuser {
            (true, true, true, true, true, true)
        } else {
            match current_user.permission_for(entry.meta.key) {
                Some(p) => (
                    p.can_create,
                    p.can_read,
                    p.can_update,
                    p.can_delete,
                    p.can_update_own,
                    p.can_delete_own,
                ),
                None => (false, false, false, false, false, false),
            }
        };
    req.context.insert(ctx_perm::CAN_CREATE, &can_create);
    req.context.insert(ctx_perm::CAN_READ, &can_read);
    req.context.insert(ctx_perm::CAN_UPDATE, &can_update);
    req.context.insert(ctx_perm::CAN_DELETE, &can_delete);
    req.context
        .insert(ctx_perm::CAN_UPDATE_OWN, &can_update_own);
    req.context
        .insert(ctx_perm::CAN_DELETE_OWN, &can_delete_own);
}

fn check_write_access(user: &CurrentUser, resource_key: &str) -> bool {
    user.is_superuser
        || user
            .permission_for(resource_key)
            .is_some_and(|p| p.can_create || p.can_update || p.can_delete)
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
        return Err(Box::new(AppError::new(ErrorContext::generic(
            StatusCode::FORBIDDEN,
            t("csrf.invalid_or_missing").as_ref(),
        ))));
    }
    Ok(())
}
