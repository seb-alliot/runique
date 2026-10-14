//! `GET {admin}/next/{resource}/{id}/detail`: one row of a model.
use super::{respond, unknown_resource};
use crate::admin::refonte_admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::auth::session::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use std::sync::Arc;

pub(crate) async fn detail_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let result = model.detail(&req.engine.db, &user, &id).await;
    respond(result, &req, &admin).await
}
