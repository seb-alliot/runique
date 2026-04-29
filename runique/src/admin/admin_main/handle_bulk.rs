use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::history;
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::{AppResult, StrMap},
    trad::t,
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

    match body.get("bulk_action").map(String::as_str).unwrap_or("") {
        "delete" => handle_bulk_delete(req, entry, ids, state, resource_key, current_user).await,
        "group_set" => {
            handle_group_set(req, entry, ids, body, state, resource_key, current_user).await
        }
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
