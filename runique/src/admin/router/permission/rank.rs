//! `POST {admin}/{resource}/rank`: gives accounts a rank, through the gate's
//! own operation. A rank is never a form field.
use crate::{
    admin::{
        builder::{
            bulk_form::{BULK_ACTION, BULK_IDS, BulkForm},
            registered::AdminError,
            site::AdminSite,
        },
        constants::admin_form,
        router::{account::summary::rank_label, failure, routes::AdminState, unknown_resource},
    },
    auth::{
        current_user::CurrentUser,
        gate::{Refusal, rank::set_rank, refusal::RefusalReason},
        permissions::rank::Rang,
        user,
    },
    forms::{traits::runique_form::FormAccess, validation_form::ValidationForm},
    http::request::Request,
    schema::pk::Pk,
    translation::{t, tf},
    utils::aliases::AppResult,
};
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use sea_orm::DbErr;
use std::sync::Arc;

/// The rank a form value names; the superuser's is never given.
pub(crate) fn parse_rank(value: &str) -> Option<Rang> {
    match value {
        "lambda" => Some(Rang::Lambda),
        "admin" => Some(Rang::Admin),
        "dirigeant" => Some(Rang::Dirigeant),
        _ => None,
    }
}

/// The ranks `actor` may give, as `(value, label)`, lowest first.
pub(crate) fn rank_choices(actor: Rang) -> Vec<(&'static str, String)> {
    [
        ("lambda", Rang::Lambda),
        ("admin", Rang::Admin),
        ("dirigeant", Rang::Dirigeant),
    ]
    .into_iter()
    .filter(|(_, rang)| actor >= Rang::Dirigeant && *rang < actor)
    .map(|(value, rang)| {
        let label = rank_label(rang);
        (value, t(label).into_owned())
    })
    .collect()
}

pub(crate) async fn rank_view(
    Path(resource): Path<String>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    if !model.is_account_table() {
        let not_found = AdminError::Refused(Refusal::new(RefusalReason::NotFound));
        return failure(not_found, &req, &admin).await;
    }
    let form = req.form::<BulkForm<user::Entity>>(admin_form::RANK);
    let form = ValidationForm::try_form(form, &req).await;
    let value = |name: &str| match &form {
        Ok(form) => form
            .forms()
            .fields
            .get(name)
            .map(|f| f.value().to_string())
            .unwrap_or_default(),
        Err(_) => String::new(),
    };
    let ids: Option<Vec<Pk>> = value(BULK_IDS)
        .split(',')
        .filter(|id| !id.trim().is_empty())
        .map(|id| id.trim().parse::<Pk>().ok())
        .collect();
    let to = parse_rank(&value(BULK_ACTION));

    let back = match ids.as_deref() {
        Some([one]) => admin.paths.detail_of(&site, model.key(), &one.to_string()),
        _ => admin.paths.list_of(model.key()),
    };
    let outcome = match (form.is_ok(), ids, to) {
        (true, Some(ids), Some(to)) => set_rank(&user, &ids, to, &req.engine.db).await,
        _ => Err(DbErr::Custom(
            Refusal::new(RefusalReason::NoRight).message(),
        )),
    };
    match outcome {
        Ok(n) => {
            req.notices
                .success(tf(runique_i18n::keys::admin::rank::info::SUCCESS, &[n]))
                .await
        }
        Err(DbErr::Custom(message)) => req.notices.error(message).await,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    }
    Ok(Redirect::to(&back).into_response())
}
