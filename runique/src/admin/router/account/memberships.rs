//! `POST {admin}/{resource}/{id}/groups`: the groups an account belongs to,
//! among those the viewer manages, then back to the account's page.
use crate::{
    admin::{
        builder::{confirm_form::ConfirmForm, registered::AdminError, site::AdminSite},
        constants::admin_form,
        router::{failure, routes::AdminState, unknown_resource},
    },
    auth::{
        current_user::CurrentUser,
        gate::{Refusal, membership::set_memberships, refusal::RefusalReason},
        user,
    },
    forms::validation_form::ValidationForm,
    http::request::Request,
    schema::pk::Pk,
    translation::t,
    utils::aliases::AppResult,
};
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::DbErr;
use std::sync::Arc;

/// The box of group `id` in the account's page.
pub(crate) fn group_box(id: i32) -> String {
    format!("g__{id}")
}

/// The groups ticked in the form, or `None` when a box isn't a group's
/// (forged): refused whole.
fn ticked_groups(req: &Request) -> Option<Vec<i32>> {
    req.prisme
        .data
        .keys()
        .filter_map(|name| name.strip_prefix("g__"))
        .map(|id| id.parse::<i32>().ok())
        .collect()
}

pub(crate) async fn memberships_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
    let Ok(target) = id.parse::<Pk>() else {
        return failure(not_found(), &req, &admin).await;
    };
    if !model.is_account_table() {
        return failure(not_found(), &req, &admin).await;
    }
    let back = admin.paths.detail_of(&site, model.key(), &id);

    let form = req.form::<ConfirmForm<user::Entity>>(admin_form::GROUPS);
    let message = match ValidationForm::try_form(form, &req).await {
        Err(_) => Err(t(runique_i18n::keys::csrf::error::INVALID_OR_MISSING).into_owned()),
        Ok(_) => match ticked_groups(&req) {
            None => Err(Refusal::new(RefusalReason::NoRight).message()),
            Some(groups) => set_memberships(&user, target, &groups, &req.engine.db)
                .await
                .map_err(|e| match e {
                    DbErr::Custom(message) => message,
                    other => {
                        tracing::error!(error = %other, "account groups write failed");
                        Refusal::new(RefusalReason::WriteFailed).message()
                    }
                }),
        },
    };
    match message {
        Ok(()) => {
            req.notices
                .success(t(runique_i18n::keys::admin::edit::info::SUCCESS))
                .await
        }
        Err(message) => req.notices.error(message).await,
    }
    Ok(Redirect::to(&back).into_response())
}
