//! `GET {admin}/{resource}/{id}/detail`: one row of a model.
use super::{chrome, failure, unknown_resource};
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
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
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let entry = match model.detail(&req.engine.db, &user, &id).await {
        Ok(entry) => entry,
        Err(error) => return failure(error, &req, &admin).await,
    };
    let key = model.key();

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "detail");
    insert_admin_messages(&mut req.context, "list");
    req = req
        .insert(
            "resource",
            serde_json::json!({ "key": key, "title": model.title() }),
        )
        .insert("resource_key", key)
        .insert("resource_base", format!("{}/{key}", admin.config.prefix))
        .insert("current_page", "detail")
        .insert("object_id", &id)
        .insert("entry", &entry)
        .insert("rich_fields", Vec::<String>::new())
        // Reading only, until writing goes through the gate: no action is offered.
        .insert("can_update", false)
        .insert("can_reset_password", false);
    req.render(admin.config.templates.detail.resolve())
}
