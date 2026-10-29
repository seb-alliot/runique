//! `GET/POST {admin}/groupes/{id}/transfer`: a group handed to another
//! responsable. The GET shows what the transfer does (the rights the members
//! lose), writing nothing; the POST makes it, for those losses only.
use crate::admin::builder::confirm_form::ConfirmForm;
use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{ResourceRef, TransferContext, TransferLine, put};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::auth::gate::Refusal;
use crate::auth::gate::refusal::RefusalReason;
use crate::auth::gate::transfer::{Transfer, plan, transfer_group};
use crate::auth::permissions::groupe;
use crate::context::template::Request;
use crate::forms::field::FormAccess;
use crate::forms::validation_form::ValidationForm;
use crate::utils::aliases::{AppResult, StrMap};
use crate::utils::constante::admin_key::admin_form;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

/// The hidden field carrying the losses the confirmation page showed.
const SHOWN: &str = "shown";
/// The receiver, in the detail's selector and the confirmation form.
const TO: &str = "to";

/// The box of the right at place `i` in the confirmation's list.
fn keep_box(i: usize) -> String {
    format!("keep__{i}")
}

/// The places ticked, or `None` when a box isn't one (forged): refused whole.
fn ticked(req: &Request) -> Option<std::collections::HashSet<usize>> {
    req.prisme
        .data
        .keys()
        .filter_map(|name| name.strip_prefix("keep__"))
        .map(|i| i.parse::<usize>().ok())
        .collect()
}

/// A right as the confirmation names it: the table's, or a column's.
fn right_label(line: &crate::auth::gate::transfer::Line, site: &AdminSite) -> (String, String) {
    let table = site
        .model(&line.resource_key)
        .map_or_else(|| line.resource_key.clone(), |m| m.title());
    let right = if line.colonne.is_empty() {
        t(&format!("admin.rights.right_{}", line.right)).into_owned()
    } else {
        format!(
            "{} : {}",
            line.colonne,
            t(&format!("admin.rights.col_{}", line.right))
        )
    };
    (table, right)
}

/// A `DbErr` from the gate as the message shown.
fn message(e: sea_orm::DbErr) -> String {
    match e {
        sea_orm::DbErr::Custom(message) => message,
        other => {
            tracing::error!(error = %other, "group transfer failed");
            Refusal::new(RefusalReason::WriteFailed).message()
        }
    }
}

pub(crate) async fn transfer_view(
    Path((resource, id)): Path<(String, String)>,
    Query(params): Query<StrMap>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let Ok(groupe_id) = id.parse::<i32>() else {
        return unknown_resource(&resource);
    };
    if model.key() != resource_key::GROUPES {
        return unknown_resource(&resource);
    }
    let paths = &admin.paths;
    let back = paths.detail_of(&site, model.key(), &id);
    let here = paths.transfer_of(model.key(), &id);

    if req.method == axum::http::Method::POST {
        let mut form = req.form::<ConfirmForm<groupe::Entity>>();
        form.get_form_mut().set_name(admin_form::TRANSFER);
        let field = |name: &str| req.prisme.data.get(name).cloned().unwrap_or_default();
        let (to, shown) = (field(TO), field(SHOWN));
        let outcome = match ValidationForm::try_form(form, &req, admin_form::TRANSFER).await {
            Err(_) => Err(t("csrf.invalid_or_missing").into_owned()),
            Ok(_) => match (to.parse::<Pk>(), ticked(&req)) {
                (Err(_), _) => Err(t("admin.transfer.unsuitable").into_owned()),
                (_, None) => Err(Refusal::new(RefusalReason::NoRight).message()),
                (Ok(to), Some(keep)) => {
                    transfer_group(&user, groupe_id, to, &shown, &keep, &req.engine.db)
                        .await
                        .map_err(message)
                }
            },
        };
        return match outcome {
            Ok(Transfer::Done) => {
                req.notices.success(t("admin.transfer.success")).await;
                Ok(Redirect::to(&back).into_response())
            }
            Ok(Transfer::Changed) => {
                req.notices.warning(t("admin.transfer.changed")).await;
                let again = format!("{here}?{TO}={}", urlencoding::encode(&to));
                Ok(Redirect::to(&again).into_response())
            }
            Err(message) => {
                req.notices.error(message).await;
                Ok(Redirect::to(&back).into_response())
            }
        };
    }

    let Some(to) = params.get(TO).and_then(|to| to.parse::<Pk>().ok()) else {
        req.notices.error(t("admin.transfer.unsuitable")).await;
        return Ok(Redirect::to(&back).into_response());
    };
    let plan = match plan(&user, groupe_id, to, &req.engine.db).await {
        Ok(plan) => plan,
        Err(sea_orm::DbErr::Custom(message)) => {
            req.notices.error(message).await;
            return Ok(Redirect::to(&back).into_response());
        }
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };

    chrome(
        &mut req,
        &admin,
        &site,
        &user,
        model.key(),
        crate::admin::template::Page::Transfer,
    );
    insert_admin_messages(&mut req.context, "transfer");
    put(
        &mut req,
        &TransferContext {
            resource: ResourceRef::new(model.key(), model.title()),
            current_page: "transfer",
            list_url: paths.list_of(model.key()),
            detail_url: back,
            form_action: here,
            group: plan.group.nom,
            from: plan.from,
            to: plan.to.username,
            to_id: to.to_string(),
            lines: plan
                .lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let (table, right) = right_label(line, &site);
                    TransferLine {
                        name: keep_box(i),
                        table,
                        right,
                        // The defaults (decided): a group handed over sideways
                        // loses what its new responsable lacks until ticked.
                        kept: false,
                        keepable: line.keepable,
                    }
                })
                .collect(),
            shown: plan.losses,
        },
    )?;
    req.render("admin/transfer.html")
}
