//! `POST {admin}/{resource}/rank`: gives accounts a rank, through the gate's
//! own operation. A rank is never a form field.
use super::{detail_url, failure, link, unknown_resource};
use crate::admin::builder::bulk_form::{BULK_ACTION, BULK_IDS, BulkForm};
use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::auth::current_user::CurrentUser;
use crate::auth::gate::Refusal;
use crate::auth::gate::rank::set_rank;
use crate::auth::gate::refusal::RefusalReason;
use crate::auth::permissions::rang::Rang;
use crate::auth::user;
use crate::context::template::Request;
use crate::forms::field::FormAccess;
use crate::forms::validation_form::ValidationForm;
use crate::utils::aliases::AppResult;
use crate::utils::constante::admin_key::{admin_form, admin_route};
use crate::utils::pk::Pk;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
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
pub(crate) fn rank_choices(actor: Rang) -> Vec<(&'static str, &'static str)> {
    [
        ("lambda", "Compte du site", Rang::Lambda),
        ("admin", "Admin", Rang::Admin),
        ("dirigeant", "Dirigeant", Rang::Dirigeant),
    ]
    .into_iter()
    .filter(|(_, _, rang)| actor >= Rang::Dirigeant && *rang < actor)
    .map(|(value, label, _)| (value, label))
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
    let mut form = req.form::<BulkForm<user::Entity>>();
    form.get_form_mut().set_name(admin_form::RANK);
    let form = ValidationForm::try_form(form, &req, admin_form::RANK).await;
    let value = |name: &str| match &form {
        Ok(form) => form
            .get_form()
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
        Some([one]) => detail_url(&req, &site, model.key(), &one.to_string()),
        _ => link(&req, admin_route::LIST, &[("resource", model.key())]),
    };
    let outcome = match (form.is_ok(), ids, to) {
        (true, Some(ids), Some(to)) => set_rank(&user, &ids, to, &req.engine.db).await,
        _ => Err(sea_orm::DbErr::Custom(
            Refusal::new(RefusalReason::NoRight).message(),
        )),
    };
    match outcome {
        Ok(n) => req.notices.success(format!("Rang modifié ({n})")).await,
        Err(sea_orm::DbErr::Custom(message)) => req.notices.error(message).await,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    }
    Ok(Redirect::to(&back).into_response())
}
