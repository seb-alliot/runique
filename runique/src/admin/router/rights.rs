//! `GET/POST {admin}/users/{id}/rights`: the grid of an admin's own rights,
//! ticked by a dirigeant or the superuser, table by table, column by column.
use super::context::{GridBox, GridColumn, GridTable, ResourceRef, RightsContext, put};
use super::{chrome, detail_url, failure, unknown_resource};
use crate::admin::builder::confirm_form::ConfirmForm;
use crate::admin::builder::registered::AdminError;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
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
const TABLE_LABELS: [&str; 7] = [
    "Créer",
    "Lister / lire",
    "Lire les siennes",
    "Modifier",
    "Modifier les siennes",
    "Supprimer",
    "Supprimer les siennes",
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

/// The grid of every model, ticked from `rows` (the admin's rows now).
fn grid(site: &AdminSite, rows: &[groupes_droits::Model]) -> Vec<GridTable> {
    site.models()
        .map(|model| {
            let key = model.key();
            let row = |column: &str| {
                rows.iter()
                    .find(|r| r.resource_key == key && r.colonne == column)
            };
            let table_row = row("");
            GridTable {
                key,
                title: model.title(),
                table: TABLE_RIGHTS
                    .iter()
                    .zip(TABLE_LABELS)
                    .map(|(right, label)| {
                        (
                            label,
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
                            read: b("read", col_row.is_some_and(|r| r.can_read)),
                            create: b("create", col_row.is_some_and(|r| r.can_create)),
                            update: b("update", col_row.is_some_and(|r| r.can_update)),
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

/// The grid sent back, or `None` when it names a box the grid doesn't have
/// (a forged table or column, a secret): refused whole, nothing written.
fn sent_grid(site: &AdminSite, req: &Request) -> Option<Vec<TableGrant>> {
    let sent: HashSet<&str> = req
        .prisme
        .data
        .keys()
        .map(String::as_str)
        .filter(|name| name.starts_with("t__") || name.starts_with("c__"))
        .collect();
    let mut known = HashSet::new();
    let mut grants = Vec::new();
    for model in site.models() {
        let key = model.key();
        let mut ticked = |name: String| {
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
    let back = detail_url(&req, &site, key, &id);

    let mut form = req.form::<ConfirmForm<user::Entity>>();
    form.get_form_mut().set_name(admin_form::RIGHTS);
    if !req.method.is_safe() {
        let message = match ValidationForm::try_form(form, &req, admin_form::RIGHTS).await {
            Err(_) => Err(t("csrf.invalid_or_missing").into_owned()),
            Ok(_) => match sent_grid(&site, &req) {
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
    put(
        &mut req,
        &RightsContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "rights",
            object_id: id,
            account_name: account.username,
            tables: grid(&site, &rows),
            form_fields,
            cancel_url: back,
        },
    )?;
    req.render("admin/rights.html")
}
