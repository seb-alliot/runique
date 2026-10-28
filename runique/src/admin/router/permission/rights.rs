//! `GET/POST {admin}/users/{id}/rights`: the grid of an admin's own rights,
//! ticked by a dirigeant or the superuser, table by table, column by column.
use crate::admin::builder::confirm_form::ConfirmForm;
use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{
    GridBox, GridColumn, GridTable, ResourceRef, RightsContext, put,
};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::auth::current_user::CurrentUser;
use crate::auth::gate::Refusal;
use crate::auth::gate::refusal::RefusalReason;
use crate::auth::gate::rights_grid::{
    ColumnGrant, TableGrant, TableRights, may_set, set_admin_rights,
};
use crate::auth::permissions::groupes_droits;
use crate::auth::user::{self, BuiltinUserEntity};
use crate::context::template::Request;
use crate::forms::field::FormAccess;
use crate::forms::validation_form::ValidationForm;
use crate::utils::aliases::AppResult;
use crate::utils::constante::admin_key::admin_form;
use crate::utils::pk::Pk;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashSet;
use std::sync::Arc;

/// The table rights, as the grid shows and sends them.
const TABLE_RIGHTS: [&str; 7] = [
    "create",
    "read",
    "read_own",
    "update",
    "update_own",
    "delete",
    "delete_own",
];
/// The translation key of each table right's label, in `TABLE_RIGHTS`' order.
const TABLE_LABELS: [&str; 7] = [
    "admin.rights.right_create",
    "admin.rights.right_read",
    "admin.rights.right_read_own",
    "admin.rights.right_update",
    "admin.rights.right_update_own",
    "admin.rights.right_delete",
    "admin.rights.right_delete_own",
];

fn table_box(key: &str, right: &str) -> String {
    format!("t__{key}__{right}")
}

fn column_box(key: &str, column: &str, right: &str) -> String {
    format!("c__{key}__{column}__{right}")
}

fn table_flag(row: &groupes_droits::Model, right: &str) -> bool {
    match right {
        "create" => row.can_create,
        "read" => row.can_read,
        "read_own" => row.can_read_own,
        "update" => row.can_update,
        "update_own" => row.can_update_own,
        "delete" => row.can_delete,
        _ => row.can_delete_own,
    }
}

/// Whether `model` belongs in the grid: every model for an admin, the
/// developer's alone for a group (no group holds a right on the framework's).
fn offered(
    model: &dyn crate::admin::builder::registered::RegisteredModel,
    for_group: bool,
) -> bool {
    !for_group || !crate::utils::constante::resource_key::FRAMEWORK_KEYS.contains(&model.key())
}

/// The grid of every model offered, ticked from `rows` (the holder's rows now).
fn grid(site: &AdminSite, rows: &[groupes_droits::Model], for_group: bool) -> Vec<GridTable> {
    site.models()
        .filter(|model| offered(*model, for_group))
        .map(|model| {
            let key = model.key();
            let row = |column: &str| {
                rows.iter()
                    .find(|r| r.resource_key == key && r.colonne == column)
            };
            let table_row = row("");
            // On an admin's grid, the groups offer their floor alone.
            let allowed = |right: &str| {
                for_group
                    || key != crate::utils::constante::resource_key::GROUPES
                    || crate::auth::gate::rights_grid::ADMIN_GROUP_RIGHTS.contains(&right)
            };
            GridTable {
                key,
                title: model.title(),
                table: TABLE_RIGHTS
                    .iter()
                    .zip(TABLE_LABELS)
                    .filter(|(right, _)| allowed(right))
                    .map(|(right, label)| {
                        (
                            crate::utils::trad::t(label).into_owned(),
                            GridBox {
                                name: table_box(key, right),
                                ticked: table_row.is_some_and(|r| table_flag(r, right)),
                            },
                        )
                    })
                    .collect(),
                columns: model
                    .grid_columns()
                    .into_iter()
                    .map(|column| {
                        let col_row = row(column);
                        let b = |right: &str, ticked: bool| GridBox {
                            name: column_box(key, column, right),
                            ticked,
                        };
                        GridColumn {
                            column,
                            read: Some(b("read", col_row.is_some_and(|r| r.can_read))),
                            create: Some(b("create", col_row.is_some_and(|r| r.can_create))),
                            update: Some(b("update", col_row.is_some_and(|r| r.can_update))),
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

/// The grid sent back, or `None` when it names a box the grid doesn't have
/// (a forged table or column, a secret): refused whole, nothing written.
fn sent_grid(site: &AdminSite, req: &Request, for_group: bool) -> Option<Vec<TableGrant>> {
    let sent: HashSet<&str> = req
        .prisme
        .data
        .keys()
        .map(String::as_str)
        .filter(|name| name.starts_with("t__") || name.starts_with("c__"))
        .collect();
    let mut known = HashSet::new();
    let mut grants = Vec::new();
    for model in site.models().filter(|model| offered(*model, for_group)) {
        let key = model.key();
        // A box the grid doesn't offer stays unknown: sent anyway, it refuses
        // the whole save (the groups' floor on an admin's grid).
        let floor = !for_group && key == crate::utils::constante::resource_key::GROUPES;
        let mut ticked = |name: String| {
            let right = name.rsplit("__").next().unwrap_or_default();
            if floor
                && name.starts_with("t__")
                && !crate::auth::gate::rights_grid::ADMIN_GROUP_RIGHTS.contains(&right)
            {
                return false;
            }
            let on = sent.contains(name.as_str());
            known.insert(name);
            on
        };
        let table = TableRights {
            create: ticked(table_box(key, "create")),
            read: ticked(table_box(key, "read")),
            read_own: ticked(table_box(key, "read_own")),
            update: ticked(table_box(key, "update")),
            update_own: ticked(table_box(key, "update_own")),
            delete: ticked(table_box(key, "delete")),
            delete_own: ticked(table_box(key, "delete_own")),
        };
        let columns = model
            .grid_columns()
            .into_iter()
            .map(|column| ColumnGrant {
                column: column.to_string(),
                read: ticked(column_box(key, column, "read")),
                create: ticked(column_box(key, column, "create")),
                update: ticked(column_box(key, column, "update")),
            })
            .collect();
        grants.push(TableGrant {
            resource_key: key.to_string(),
            table,
            columns,
        });
    }
    sent.iter()
        .all(|name| known.contains(*name))
        .then_some(grants)
}

pub(crate) async fn rights_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
    if model.key() == crate::utils::constante::resource_key::GROUPES {
        return group_rights(id, site, admin, user, req).await;
    }
    if !model.is_account_table() {
        return failure(not_found(), &req, &admin).await;
    }
    let Ok(target) = id.parse::<Pk>() else {
        return failure(not_found(), &req, &admin).await;
    };
    let Some(account) = BuiltinUserEntity::find_by_id(&req.engine.db, target).await else {
        return failure(not_found(), &req, &admin).await;
    };
    if may_set(&user, &account).is_err() {
        return failure(not_found(), &req, &admin).await;
    }
    let key = model.key();
    let back = admin.paths.detail_of(&site, key, &id);

    let mut form = req.form::<ConfirmForm<user::Entity>>();
    form.get_form_mut().set_name(admin_form::RIGHTS);
    if !req.method.is_safe() {
        let message = match ValidationForm::try_form(form, &req, admin_form::RIGHTS).await {
            Err(_) => Err(t("csrf.invalid_or_missing").into_owned()),
            Ok(_) => match sent_grid(&site, &req, false) {
                None => Err(Refusal::new(RefusalReason::NoRight).message()),
                Some(grants) => set_admin_rights(&user, target, &grants, &req.engine.db)
                    .await
                    .map_err(|e| match e {
                        sea_orm::DbErr::Custom(message) => message,
                        other => {
                            tracing::error!(error = %other, "admin rights grid write failed");
                            Refusal::new(RefusalReason::WriteFailed).message()
                        }
                    }),
            },
        };
        match message {
            Ok(()) => req.notices.success(t("admin.edit.success")).await,
            Err(message) => req.notices.error(message).await,
        }
        return Ok(Redirect::to(&back).into_response());
    }

    let rows = match groupes_droits::Entity::find()
        .filter(groupes_droits::Column::UserId.eq(target))
        .all(&req.engine.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    let form_fields = serde_json::to_value(form.get_form()).unwrap_or_default();
    chrome(&mut req, &admin, &site, &user, key);
    crate::admin::trad::insert_admin_messages(&mut req.context, "rights");
    put(
        &mut req,
        &RightsContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "rights",
            list_url: admin.paths.list_of(key),
            form_action: admin.paths.rights_of(key, &id),
            object_id: id,
            account_name: account.username,
            tables: grid(&site, &rows, false),
            is_group: false,
            group_choices: Vec::new(),
            group_select_url: None,
            form_fields,
            cancel_url: back,
        },
    )?;
    req.render("admin/rights.html")
}

/// `tables` cut down to what `holder`, an admin's own grid, lets him give:
/// a table he holds no right on is left out, name included, and so is each
/// right he lacks (his full right covering an `_own` one).
fn within_holder(
    tables: Vec<GridTable>,
    holder: &[crate::auth::permissions::Permission],
) -> Vec<GridTable> {
    tables
        .into_iter()
        .filter_map(|mut table| {
            let h = holder.iter().find(|p| p.resource_key == table.key)?;
            let holds = |right: &str| match right {
                "create" => h.can_create,
                "read" => h.can_read,
                "read_own" => h.can_read_own || h.can_read,
                "update" => h.can_update,
                "update_own" => h.can_update_own || h.can_update,
                "delete" => h.can_delete,
                _ => h.can_delete_own || h.can_delete,
            };
            table.table = table
                .table
                .into_iter()
                .zip(TABLE_RIGHTS)
                .filter(|(_, right)| holds(right))
                .map(|(entry, _)| entry)
                .collect();
            table.columns = table
                .columns
                .into_iter()
                .filter_map(|mut column| {
                    let hc = h.colonnes.iter().find(|c| c.colonne == column.column)?;
                    column.read = column.read.filter(|_| hc.can_read);
                    column.create = column.create.filter(|_| hc.can_create);
                    column.update = column.update.filter(|_| hc.can_update);
                    Some(column)
                })
                .collect();
            (!table.table.is_empty() || !table.columns.is_empty()).then_some(table)
        })
        .collect()
}

/// A group's rights: what its members may do on the developer's tables.
/// Same page as an admin's grid, written by `set_group_rights`.
async fn group_rights(
    id: String,
    site: Arc<AdminSite>,
    admin: Arc<AdminState>,
    user: CurrentUser,
    mut req: Request,
) -> AppResult<Response> {
    use crate::auth::gate::rights_grid::{may_set_group, set_group_rights};
    use crate::auth::permissions::groupe;
    let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
    let Ok(group_id) = id.parse::<i32>() else {
        return failure(not_found(), &req, &admin).await;
    };
    let group = match groupe::Entity::find_by_id(group_id)
        .one(&req.engine.db)
        .await
    {
        Ok(Some(group)) => group,
        Ok(None) => return failure(not_found(), &req, &admin).await,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    if may_set_group(&user, &group).is_err() {
        return failure(not_found(), &req, &admin).await;
    }
    let key = crate::utils::constante::resource_key::GROUPES;
    let back = admin.paths.detail_of(&site, key, &id);

    let mut form = req.form::<ConfirmForm<groupe::Entity>>();
    form.get_form_mut().set_name(admin_form::RIGHTS);
    if !req.method.is_safe() {
        let message = match ValidationForm::try_form(form, &req, admin_form::RIGHTS).await {
            Err(_) => Err(t("csrf.invalid_or_missing").into_owned()),
            Ok(_) => match sent_grid(&site, &req, true) {
                None => Err(Refusal::new(RefusalReason::NoRight).message()),
                Some(grants) => set_group_rights(&user, group_id, &grants, &req.engine.db)
                    .await
                    .map_err(|e| match e {
                        sea_orm::DbErr::Custom(message) => message,
                        other => {
                            tracing::error!(error = %other, "group rights grid write failed");
                            Refusal::new(RefusalReason::WriteFailed).message()
                        }
                    }),
            },
        };
        match message {
            Ok(()) => req.notices.success(t("admin.edit.success")).await,
            Err(message) => req.notices.error(message).await,
        }
        return Ok(Redirect::to(&back).into_response());
    }

    let rows = match groupes_droits::Entity::find()
        .filter(groupes_droits::Column::GroupeId.eq(group_id))
        .all(&req.engine.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    let title = site.model(key).map(|m| m.title()).unwrap_or_default();
    // An admin is offered only what he may give: neither a table nor a
    // column he holds no right on shows, not even by its name.
    let mut tables = grid(&site, &rows, true);
    if user.rang == crate::auth::permissions::rang::Rang::Admin {
        tables = within_holder(tables, &user.droits);
    }
    let group_choices = match managed_groups(&user, &req.engine.db).await {
        Ok(groups) => groups
            .into_iter()
            .map(|g| (g.id.to_string(), g.nom))
            .collect(),
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    let form_fields = serde_json::to_value(form.get_form()).unwrap_or_default();
    chrome(&mut req, &admin, &site, &user, key);
    crate::admin::trad::insert_admin_messages(&mut req.context, "rights");
    put(
        &mut req,
        &RightsContext {
            resource: ResourceRef::new(key, title),
            resource_key: key,
            current_page: "rights",
            list_url: admin.paths.list_of(key),
            form_action: admin.paths.rights_of(key, &id),
            object_id: id,
            account_name: group.nom,
            tables,
            is_group: true,
            group_choices,
            group_select_url: Some(admin.paths.group_select.clone()),
            form_fields,
            cancel_url: back,
        },
    )?;
    req.render("admin/rights.html")
}

/// The groups `user` may set, by name (see the gate's `managed_groups`).
pub(crate) async fn managed_groups(
    user: &CurrentUser,
    db: &crate::utils::aliases::ADb,
) -> Result<Vec<crate::auth::permissions::groupe::Model>, sea_orm::DbErr> {
    crate::auth::gate::membership::managed_groups(user, db).await
}

/// `GET {admin}/groupes/rights?groupe={id}`: the group picked in the rights
/// page's selector. Changes nothing: it sends to that group's grid, which the
/// gate checks again; a group the account may not set is refused like a
/// missing one, so its existence tells nothing.
pub(crate) async fn group_select_view(
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    axum::extract::Query(params): axum::extract::Query<crate::utils::aliases::StrMap>,
    req: Request,
) -> AppResult<Response> {
    use crate::auth::gate::rights_grid::may_set_group;
    use crate::auth::permissions::groupe;
    let not_found = || AdminError::Refused(Refusal::new(RefusalReason::NotFound));
    // Without a group, the first one the account may set, by name: the
    // list's "rights" shortcut.
    let group_id = match params.get("groupe") {
        Some(id) => match id.parse::<i32>() {
            Ok(id) => id,
            Err(_) => return failure(not_found(), &req, &admin).await,
        },
        None => match managed_groups(&user, &req.engine.db).await {
            Ok(groups) => match groups.first() {
                Some(group) => group.id,
                None => return failure(not_found(), &req, &admin).await,
            },
            Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
        },
    };
    let group = match groupe::Entity::find_by_id(group_id)
        .one(&req.engine.db)
        .await
    {
        Ok(Some(group)) => group,
        Ok(None) => return failure(not_found(), &req, &admin).await,
        Err(e) => return failure(AdminError::Db(e), &req, &admin).await,
    };
    if may_set_group(&user, &group).is_err() {
        return failure(not_found(), &req, &admin).await;
    }
    let url = admin.paths.rights_of(
        crate::utils::constante::resource_key::GROUPES,
        &group_id.to_string(),
    );
    Ok(Redirect::to(&url).into_response())
}
