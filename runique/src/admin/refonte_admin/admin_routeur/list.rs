//! `GET {admin}/next/{resource}/list`: one page of a model's list.
use super::{respond, unknown_resource};
use crate::admin::refonte_admin::builder::list::ListQuery;
use crate::admin::refonte_admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::auth::session::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::{AppResult, StrMap};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use std::sync::Arc;

pub(crate) async fn list_view(
    Path(resource): Path<String>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    Query(params): Query<StrMap>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let query = ListQuery::from_params(params);
    let result = model
        .list(&req.engine.db, &user, &query, admin.config.page_size)
        .await;
    respond(result, &req, &admin).await
}
