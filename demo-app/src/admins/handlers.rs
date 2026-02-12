// ═══════════════════════════════════════════════════════════════
// AUTO-GÉNÉRÉ par Runique daemon — NE PAS MODIFIER MANUELLEMENT
// Ressources : users
// ═══════════════════════════════════════════════════════════════
#![allow(unused_imports, dead_code)]

use runique::prelude::*;
use runique::utils::aliases::StrMap;
use std::sync::Arc;

use crate::forms::RegisterForm;
use crate::models::users;

// ───────────── Handler users_list ─────────────
pub async fn users_list(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if req.is_get() {
        let entries = <users::Model as ModelTrait>::Entity::find()
            .all(&*req.engine.db)
            .await?;

        context_update!(req => {
            "resource_key" => "users",
            "form_fields" => &form,
            "entries" => &entries,
            "total" => entries.len()
        });
        return req.render("admin/list");
    }

    if req.is_post() {
        if form.is_valid().await {
            form.save(&req.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(req.notices => "Entrée créée avec succès !");
            return Ok(
                Redirect::to(&format!("/{}/users/list", admin.config.prefix)).into_response(),
            );
        } else {
            context_update!(req => {
                "resource_key" => "users",
                "form_fields" => &form,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs")
            });
            return req.render("admin/list");
        }
    }

    req.render("admin/list")
}

// ───────────── Handler users_create ─────────────
pub async fn users_create(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if req.is_get() {
        context_update!(req => {
            "resource_key" => "users",
            "form_fields" => &form,
            "is_edit" => false
        });
        return req.render("admin/form");
    }

    if req.is_post() {
        if form.is_valid().await {
            form.save(&req.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(req.notices => "Entrée créée avec succès !");
            return Ok(
                Redirect::to(&format!("/{}/users/list", admin.config.prefix)).into_response(),
            );
        } else {
            context_update!(req => {
                "resource_key" => "users",
                "form_fields" => &form,
                "is_edit" => false,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs")
            });
            return req.render("admin/form");
        }
    }

    req.render("admin/form")
}

// ───────────── Handler users_edit ─────────────
pub async fn users_edit(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path(id): Path<i32>,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let entry = <users::Model as ModelTrait>::Entity::find_by_id(id)
        .one(&*req.engine.db)
        .await?
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Entry not found"))))?;

    if req.is_get() {
        // Convertir Model → StrMap via JSON pour pré-remplir le form
        let entry_json = serde_json::to_value(&entry)
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

        let mut form_data = StrMap::new();
        if let Some(obj) = entry_json.as_object() {
            for (k, v) in obj {
                form_data.insert(k.clone(), v.to_string().trim_matches('"').to_string());
            }
        }

        // Remplir le form existant avec les données de l'entry
        form.get_form_mut().fill(&form_data);

        context_update!(req => {
            "resource_key" => "users",
            "form_fields" => &form,
            "is_edit" => true,
            "object_id" => id,
            "entry" => &entry
        });
        return req.render("admin/form");
    }

    if req.is_post() {
        if form.is_valid().await {
            form.save(&req.engine.db).await.map_err(|err| {
                form.get_form_mut().database_error(&err);
                AppError::from(err)
            })?;
            success!(req.notices => "Entrée mise à jour avec succès !");
            return Ok(
                Redirect::to(&format!("/{}/users/list", admin.config.prefix)).into_response(),
            );
        } else {
            context_update!(req => {
                "resource_key" => "users",
                "form_fields" => &form,
                "is_edit" => true,
                "object_id" => id,
                "entry" => &entry,
                "messages" => flash_now!(error => "Veuillez corriger les erreurs")
            });
            return req.render("admin/form");
        }
    }

    req.render("admin/form")
}

// ───────────── Handler users_detail ─────────────
pub async fn users_detail(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    let entry = <users::Model as ModelTrait>::Entity::find_by_id(id)
        .one(&*req.engine.db)
        .await?
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Entry not found"))))?;

    context_update!(req => {
        "resource_key" => "users",
        "entry" => &entry,
        "object_id" => id
    });

    req.render("admin/detail")
}

// ───────────── Handler users_delete ─────────────
pub async fn users_delete(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    if req.is_post() {
        let entry = <users::Model as ModelTrait>::Entity::find_by_id(id)
            .one(&*req.engine.db)
            .await?
            .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Entry not found"))))?;

        entry.delete(&*req.engine.db).await?;
        success!(req.notices => "Entrée supprimée avec succès !");
        return Ok(Redirect::to(&format!("/{}/users/list", admin.config.prefix)).into_response());
    }

    let entry = <users::Model as ModelTrait>::Entity::find_by_id(id)
        .one(&*req.engine.db)
        .await?
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Entry not found"))))?;

    context_update!(req => {
        "resource_key" => "users",
        "entry" => &entry,
        "object_id" => id
    });

    req.render("admin/delete_confirm")
}
