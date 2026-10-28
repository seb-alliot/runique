//! `POST {admin}/{resource}/bulk`: a grouped action on the list's checked
//! rows, then back to the same list page.
use crate::admin::builder::registered::BulkOutcome;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::{failure, unknown_resource};
use crate::auth::current_user::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

/// The list's query sent back by the bar, kept only when it is a plain query
/// string: it is appended to the list's own path, so it can't lead elsewhere,
/// and nothing in it can break the `Location` header.
fn return_query(req: &Request) -> Option<&str> {
    let qs = req.prisme.data.get("return_qs")?.as_str();
    qs.chars()
        .all(|c| c.is_ascii_alphanumeric() || "%&=_-.+".contains(c))
        .then_some(qs)
        .filter(|qs| !qs.is_empty())
}

pub(crate) async fn bulk_view(
    Path(resource): Path<String>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    match model.bulk(&req, &user, admin.config.page_size).await {
        Ok(BulkOutcome::Updated(n)) => {
            req.notices
                .success(format!("{n} {}", t("admin.bulk.update_success")))
                .await;
        }
        Ok(BulkOutcome::Deleted(n)) => {
            req.notices
                .success(format!("{n} {}", t("admin.bulk.delete_success")))
                .await;
        }
        Ok(BulkOutcome::Refused(message)) => req.notices.error(message).await,
        Err(error) => return failure(error, &req, &admin).await,
    }
    let list = admin.paths.list_of(model.key());
    let back = match return_query(&req) {
        Some(qs) => format!("{list}?{qs}"),
        None => list,
    };
    Ok(Redirect::to(&back).into_response())
}
