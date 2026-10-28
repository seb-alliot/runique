//! `GET {admin}/{resource}/{id}/detail`: one row of a model.
use super::context::{DetailContext, ResourceRef, put};
use super::{chrome, failure, unknown_resource};
use crate::admin::builder::site::AdminSite;
use crate::admin::router::AdminView;
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
    if site.overridden_path(&resource, AdminView::Detail).is_some() {
        return unknown_resource(&resource);
    }
    let entry = match model.detail(&req.engine.db, &user, &id).await {
        Ok(entry) => entry,
        Err(error) => return failure(error, &req, &admin).await,
    };
    let key = model.key();
    let rights = model.rights(&user);
    // The rank of an account below the viewer's, offered to change.
    let mut rank_choices = Vec::new();
    let mut current_rank = None;
    let mut rights_url = None;
    if model.is_account_table()
        && let Ok(pk) = id.parse::<crate::utils::pk::Pk>()
        && let Some(account) =
            crate::auth::user::BuiltinUserEntity::find_by_id(&req.engine.db, pk).await
    {
        let rang = crate::auth::permissions::rang::Rang::from(&account);
        if crate::auth::gate::rights_grid::may_set(&user, &account).is_ok() {
            rights_url = Some(super::link(
                &req,
                crate::utils::constante::admin_key::admin_route::RIGHTS,
                &[("resource", model.key()), ("id", &id)],
            ));
        }
        if rang < user.rang && account.id != user.id {
            rank_choices = super::rank::rank_choices(user.rang);
            current_rank = Some(match rang {
                crate::auth::permissions::rang::Rang::Lambda => "lambda",
                crate::auth::permissions::rang::Rang::Admin => "admin",
                crate::auth::permissions::rang::Rang::Dirigeant => "dirigeant",
                crate::auth::permissions::rang::Rang::Superuser => "superuser",
            });
        }
    }

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "detail");
    insert_admin_messages(&mut req.context, "list");
    put(
        &mut req,
        &DetailContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "detail",
            object_id: id,
            entry,
            rich_fields: Vec::new(),
            can_update: rights.can_update || rights.can_update_own,
            can_delete: rights.can_delete || rights.can_delete_own,
            can_reset_password: model.is_account_table() && rights.can_update,
            rank_choices,
            current_rank,
            rights_url,
        },
    )?;
    req.render(admin.config.templates.detail.resolve())
}
