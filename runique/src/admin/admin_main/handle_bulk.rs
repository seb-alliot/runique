use crate::admin::admin_main::gate::{BulkGrant, BulkOp, BulkRefusal, bulk_form, bulk_gate};
use crate::admin::admin_main::{ParentBinding, ResourcePerms, permission_denied, scope_base};
use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::history;
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::{AppResult, StrMap},
    constante::admin_context::{bulk_edit as ctx_bulk, create as ctx_create},
    trad::{current_lang, t},
};
use axum::response::{IntoResponse, Redirect, Response};
use uuid::Uuid;

/// Serializes a history summary to JSON. A serialization error is traced (not
/// swallowed) and yields `None` so the summary is omitted rather than lost silently.
///
/// Takes the map by value so [`history::redact_sensitive`] can be applied to it
/// before serialization: this is the single pass-through point for all bulk
/// paths, so no caller has to remember to filter.
fn summary_json(
    mut value: serde_json::Map<String, serde_json::Value>,
    resource_key: &str,
) -> Option<String> {
    history::redact_sensitive(&mut value);
    match serde_json::to_string(&value) {
        Ok(s) => Some(s),
        Err(e) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(
                    level,
                    resource = resource_key,
                    error = %e,
                    "history summary serialization failed — summary omitted"
                );
            }
            None
        }
    }
}

/// Loads the pre-change row used to build a bulk-action history summary. A DB
/// error is traced (not swallowed) and yields `None` so the diff is simply
/// omitted — the mutation itself still succeeds and is logged.
async fn fetch_old_for_summary(
    entry: &ResourceEntry,
    db: &crate::utils::aliases::ADb,
    cid: &str,
) -> Option<serde_json::Value> {
    let get_fn = entry.get_fn.as_ref()?;
    match get_fn(db.clone(), cid.to_string()).await {
        Ok(v) => v,
        Err(e) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(
                    level,
                    resource = entry.meta.key,
                    id = %cid,
                    error = %e,
                    "bulk summary get_fn failed — change diff omitted"
                );
            }
            None
        }
    }
}

pub(super) async fn handle_bulk_edit_get(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
    params: &StrMap,
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let ids_raw = params.get("ids").cloned().unwrap_or_default();
    let bulk_count = ids_raw.split(',').filter(|s| !s.trim().is_empty()).count();
    let form = bulk_form(req, entry, state, parent).await;
    req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
    req.context.insert(
        crate::utils::constante::admin_context::common::LANG,
        &current_lang().code(),
    );
    req.context.insert(ctx_bulk::BULK_COUNT, &bulk_count);
    req.context.insert(ctx_bulk::BULK_IDS, &ids_raw);
    req.render(state.config.templates.bulk_edit.resolve())
}

pub(super) async fn handle_bulk_action(
    req: &mut Request,
    entry: &ResourceEntry,
    body: StrMap,
    state: &super::PrototypeAdminState,
    current_user: &CurrentUser,
    perms: &ResourcePerms,
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let base = scope_base(&state.config.prefix, entry, parent);
    let list_url = format!("{base}/list");
    let Some(op) = BulkOp::parse(body.get("bulk_action").map_or("", String::as_str)) else {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown bulk action",
        ))));
    };

    let grant = match bulk_gate(req, entry, state, perms, parent, op, &body).await {
        Ok(grant) => grant,
        Err(refusal) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.bulk)
            {
                crate::runique_log!(level, resource = entry.meta.key, user = %current_user.username, refusal = ?refusal, "bulk refused");
            }
            return match refusal {
                BulkRefusal::NoSelection => {
                    req.notices
                        .warning(t("admin.bulk.no_selection").to_string())
                        .await;
                    Ok(Redirect::to(&list_url).into_response())
                }
                BulkRefusal::NothingToApply => {
                    req.notices
                        .warning(t("admin.bulk.no_field_selected").to_string())
                        .await;
                    Ok(Redirect::to(&list_url).into_response())
                }
                BulkRefusal::Forbidden | BulkRefusal::NotOffered(_) => {
                    Ok(permission_denied(&req.notices, &base).await)
                }
                BulkRefusal::OutOfScope => Err(Box::new(AppError::new(ErrorContext::not_found(
                    "Resource not found",
                )))),
                BulkRefusal::Invalid(msg) => {
                    req.notices.error(msg).await;
                    Ok(Redirect::to(&list_url).into_response())
                }
            };
        }
    };

    match op {
        BulkOp::Delete => handle_bulk_delete(req, entry, grant, current_user, &list_url).await,
        BulkOp::Update | BulkOp::GroupSet => {
            handle_bulk_update(req, entry, grant, current_user, &list_url).await
        }
    }
}

/// Writes what the gate accepted on every granted row, with one history entry each.
async fn handle_bulk_update(
    req: &mut Request,
    entry: &ResourceEntry,
    grant: BulkGrant,
    current_user: &CurrentUser,
    list_url: &str,
) -> AppResult<Response> {
    let BulkGrant { ids, data: updates } = grant;
    let update_fn = entry
        .partial_update_fn
        .as_ref()
        .or(entry.update_fn.as_ref())
        .ok_or_else(|| {
            Box::new(AppError::new(ErrorContext::not_found(
                t("admin.delete.not_found").as_ref(),
            )))
        })?;

    let batch_id = Some(Uuid::new_v4().to_string());
    let count = ids.len();
    for cid in &ids {
        let summary = if entry.get_fn.is_some() {
            fetch_old_for_summary(entry, &req.engine.db, cid)
                .await
                .and_then(|old_val| {
                    let serde_json::Value::Object(map) = &old_val else {
                        return None;
                    };
                    let changes: serde_json::Map<_, _> = updates
                        .iter()
                        .map(|(k, new_v)| {
                            let old_v = match map.get(k) {
                                Some(serde_json::Value::String(s)) => s.clone(),
                                Some(v) => v.to_string(),
                                None => String::new(),
                            };
                            (k.clone(), serde_json::json!({ "old": old_v, "new": new_v }))
                        })
                        .collect();
                    summary_json(changes, entry.meta.key)
                })
        } else {
            let map: serde_json::Map<_, _> = updates
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!({ "new": v })))
                .collect();
            summary_json(map, entry.meta.key)
        };

        match update_fn(req.engine.db.clone(), cid.clone(), updates.clone()).await {
            Ok(()) => {}
            Err(e) if crate::admin::builtin::is_unique_violation(&e) => {
                req.notices
                    .error(t("forms.unique_constraint_violated").to_string())
                    .await;
                return Ok(Redirect::to(list_url).into_response());
            }
            // A refusal from the resource's own update function, meant for the admin.
            Err(sea_orm::DbErr::Custom(msg)) => {
                req.notices.error(msg).await;
                return Ok(Redirect::to(list_url).into_response());
            }
            Err(e) => return Err(Box::new(AppError::new(ErrorContext::database(e)))),
        }
        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: cid,
                action: "edit",
                summary,
                batch_id: batch_id.clone(),
            },
        )
        .await;
    }

    req.notices
        .success(format!("{count} {}", t("admin.bulk.update_success")))
        .await;
    Ok(Redirect::to(list_url).into_response())
}

async fn handle_bulk_delete(
    req: &mut Request,
    entry: &ResourceEntry,
    grant: BulkGrant,
    current_user: &CurrentUser,
    list_url: &str,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    let batch_id = Some(Uuid::new_v4().to_string());
    let count = grant.ids.len();
    for cid in &grant.ids {
        delete_fn(req.engine.db.clone(), cid.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: cid,
                action: "delete",
                summary: None,
                batch_id: batch_id.clone(),
            },
        )
        .await;
    }

    req.notices
        .success(format!("{count} {}", t("admin.bulk.delete_success")))
        .await;
    Ok(Redirect::to(list_url).into_response())
}
