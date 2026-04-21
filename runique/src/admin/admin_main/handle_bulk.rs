use crate::admin::helper::resource_entry::ResourceEntry;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::{AppResult, StrMap},
    trad::t,
};
use axum::response::{IntoResponse, Redirect, Response};

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
    state: &super::PrototypeAdminState,
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
    state: &super::PrototypeAdminState,
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
