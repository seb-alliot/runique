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

pub(super) async fn handle_bulk_edit_get(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
    params: &StrMap,
) -> AppResult<Response> {
    let ids_raw = params.get("ids").cloned().unwrap_or_default();
    let ids: Vec<&str> = ids_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let bulk_count = ids.len();

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
        StrMap::new(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;

    // Inject a blank "no-change" placeholder on all select fields so they submit ""
    // when left untouched, which the update handler filters out.
    {
        let forms = form.get_form_mut();
        for field in forms.fields.values_mut() {
            if field.field_type() == "select" && field.placeholder().is_empty() {
                field.set_placeholder("— sans changement —");
            }
        }
    }
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
    resource_key: &str,
    current_user: &CurrentUser,
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

    let bulk_action = body.get("bulk_action").map(String::as_str).unwrap_or("");
    match bulk_action {
        "delete" => handle_bulk_delete(req, entry, ids, state, resource_key, current_user).await,
        "group_set" => {
            handle_group_set(req, entry, ids, body, state, resource_key, current_user).await
        }
        "update-submit" => {
            handle_bulk_update(req, entry, ids, body, state, resource_key, current_user).await
        }
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Unknown bulk action",
        )))),
    }
}

async fn handle_bulk_update(
    req: &mut Request,
    entry: &ResourceEntry,
    ids: Vec<String>,
    body: StrMap,
    state: &super::PrototypeAdminState,
    resource_key: &str,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    let list_url = format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        resource_key
    );

    // Only fields with non-empty values are applied
    let updates: StrMap = body
        .iter()
        .filter(|(k, v)| {
            !v.is_empty()
                && k.as_str() != "bulk_action"
                && k.as_str() != "ids"
                && k.as_str() != crate::utils::session_key::session::CSRF_TOKEN_KEY
        })
        .map(|(k, v)| (k.clone(), v.clone()))
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

    let batch_id = Some(Uuid::new_v4().to_string());
    let count = ids.len();
    for id in &ids {
        let summary = if let Some(get_fn) = &entry.get_fn {
            let old = get_fn(req.engine.db.clone(), id.clone())
                .await
                .ok()
                .flatten();
            old.and_then(|old_val| {
                if let serde_json::Value::Object(map) = &old_val {
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
                    serde_json::to_string(&changes).ok()
                } else {
                    None
                }
            })
        } else {
            let map: serde_json::Map<_, _> = updates
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!({ "new": v })))
                .collect();
            serde_json::to_string(&map).ok()
        };

        match update_fn(req.engine.db.clone(), id.clone(), updates.clone()).await {
            Ok(()) => {}
            Err(e) if is_unique_violation(&e) => {
                req.notices
                    .error(t("forms.unique_constraint_violated").to_string())
                    .await;
                return Ok(Redirect::to(&list_url).into_response());
            }
            Err(e) => return Err(Box::new(AppError::new(ErrorContext::database(e)))),
        }
        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: id,
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
    Ok(Redirect::to(&list_url).into_response())
}

fn is_unique_violation(e: &sea_orm::DbErr) -> bool {
    let msg = e.to_string();
    msg.contains("unique") || msg.contains("UNIQUE") || msg.contains("Duplicate")
}

async fn handle_group_set(
    req: &mut Request,
    entry: &ResourceEntry,
    ids: Vec<String>,
    body: StrMap,
    state: &super::PrototypeAdminState,
    resource_key: &str,
    current_user: &CurrentUser,
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

    let batch_id = Some(Uuid::new_v4().to_string());
    let count = ids.len();
    for id in &ids {
        let summary = if let Some(get_fn) = &entry.get_fn {
            let old = get_fn(req.engine.db.clone(), id.clone())
                .await
                .ok()
                .flatten();
            old.and_then(|old_val| {
                if let serde_json::Value::Object(map) = &old_val {
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
                    serde_json::to_string(&changes).ok()
                } else {
                    None
                }
            })
        } else {
            let map: serde_json::Map<_, _> = updates
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!({ "new": v })))
                .collect();
            serde_json::to_string(&map).ok()
        };

        update_fn(req.engine.db.clone(), id.clone(), updates.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: id,
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
    Ok(Redirect::to(&list_url).into_response())
}

async fn handle_bulk_delete(
    req: &mut Request,
    entry: &ResourceEntry,
    ids: Vec<String>,
    state: &super::PrototypeAdminState,
    resource_key: &str,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    let batch_id = Some(Uuid::new_v4().to_string());
    let count = ids.len();
    for id in &ids {
        delete_fn(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: id,
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
    Ok(Redirect::to(&format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        resource_key
    ))
    .into_response())
}
