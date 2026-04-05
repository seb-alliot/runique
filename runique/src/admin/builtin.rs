//! Ressources admin built-in Runique — droits et groupes.
//!
//! Injectées automatiquement dans le registre admin sans déclaration dans `admin!{}`.

use std::sync::Arc;

use axum::http::Method;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QuerySelect};

use crate::admin::dyn_form::DynForm;
use crate::admin::forms::{
    DroitAdminForm, GroupeAdminForm, UserAdminCreateForm, UserAdminEditForm,
};
use crate::admin::resource::AdminResource;
use crate::admin::resource_entry::{
    CountFn, CreateFn, DeleteFn, FormBuilder, GetFn, ListFn, ListParams, ResourceEntry, SortDir,
    UpdateFn,
};
use crate::forms::field::RuniqueForm;
use crate::utils::aliases::{ADb, ATera, StrMap};

/// Retourne les `ResourceEntry` built-in Runique (droits, groupes).
/// Appelé automatiquement dans le registre admin généré par le daemon.
pub fn builtin_resources() -> Vec<ResourceEntry> {
    vec![user_entry(), droit_entry(), groupe_entry()]
}

// ─── User ─────────────────────────────────────────────────────────────────────

fn user_entry() -> ResourceEntry {
    use crate::middleware::auth::user;
    use crate::utils::pk::UserId;

    let meta = AdminResource::new(
        "users",
        "runique::middleware::auth::user::Model",
        "UserAdminCreateForm",
        "Utilisateurs",
        vec!["admin".to_string()],
    )
    .inject_password(true);

    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = UserAdminCreateForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(UserCreateDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let edit_form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = UserAdminEditForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(UserEditDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = user::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({col} AS TEXT) = '{escaped}'")));
            }
            let rows = query
                .offset(params.offset)
                .limit(params.limit)
                .all(&*db)
                .await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb, _| Box::pin(async move { user::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<UserId>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            let row = user::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<UserId>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            user::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            let now = Some(chrono::Utc::now().naive_utc());
            user::ActiveModel {
                username: Set(data.get("username").cloned().unwrap_or_default()),
                email: Set(data.get("email").cloned().unwrap_or_default()),
                password: Set(data.get("password").cloned().unwrap_or_default()),
                is_active: Set(data
                    .get("is_active")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(true)),
                is_staff: Set(data
                    .get("is_staff")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(false)),
                is_superuser: Set(data
                    .get("is_superuser")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(false)),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&*db)
            .await
            .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<UserId>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            user::ActiveModel {
                id: Set(id),
                username: Set(data.get("username").cloned().unwrap_or_default()),
                email: Set(data.get("email").cloned().unwrap_or_default()),
                is_active: Set(data
                    .get("is_active")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(true)),
                is_staff: Set(data
                    .get("is_staff")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(false)),
                is_superuser: Set(data
                    .get("is_superuser")
                    .map(|v| v == "on" || v == "true")
                    .unwrap_or(false)),
                updated_at: Set(Some(chrono::Utc::now().naive_utc())),
                ..Default::default()
            }
            .update(&*db)
            .await
            .map(|_| ())
        })
    });

    ResourceEntry::new(meta, form_builder)
        .with_edit_form_builder(edit_form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_get_fn(get_fn)
        .with_delete_fn(delete_fn)
        .with_create_fn(create_fn)
        .with_update_fn(update_fn)
}

// ─── Droit ───────────────────────────────────────────────────────────────────

fn droit_entry() -> ResourceEntry {
    use crate::admin::permissions::droit;

    let meta = AdminResource::new(
        "droits",
        "runique::admin::permissions::droit::Model",
        "DroitAdminForm",
        "Droits",
        vec!["admin".to_string()],
    );

    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = DroitAdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DroitDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = droit::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({col} AS TEXT) = '{escaped}'")));
            }
            let rows = query
                .offset(params.offset)
                .limit(params.limit)
                .all(&*db)
                .await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _| {
        Box::pin(async move {
            use sea_orm::EntityTrait;
            droit::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            let row = droit::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            droit::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            let nom = data.get("nom").cloned().unwrap_or_default();
            droit::ActiveModel {
                nom: Set(nom),
                ..Default::default()
            }
            .insert(&*db)
            .await
            .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            let nom = data.get("nom").cloned().unwrap_or_default();
            droit::ActiveModel {
                id: Set(id),
                nom: Set(nom),
            }
            .update(&*db)
            .await
            .map(|_| ())
        })
    });

    ResourceEntry::new(meta, form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_get_fn(get_fn)
        .with_delete_fn(delete_fn)
        .with_create_fn(create_fn)
        .with_update_fn(update_fn)
}

// ─── Groupe ──────────────────────────────────────────────────────────────────

fn groupe_entry() -> ResourceEntry {
    use crate::admin::permissions::groupe;

    let meta = AdminResource::new(
        "groupes",
        "runique::admin::permissions::groupe::Model",
        "GroupeAdminForm",
        "Groupes",
        vec!["admin".to_string()],
    );

    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = GroupeAdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(GroupeDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = groupe::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({col} AS TEXT) = '{escaped}'")));
            }
            let rows = query
                .offset(params.offset)
                .limit(params.limit)
                .all(&*db)
                .await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb, _| Box::pin(async move { groupe::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            let row = groupe::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            groupe::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            let nom = data.get("nom").cloned().unwrap_or_default();
            groupe::ActiveModel {
                nom: Set(nom),
                ..Default::default()
            }
            .insert(&*db)
            .await
            .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| sea_orm::DbErr::Custom("id invalide".into()))?;
            let nom = data.get("nom").cloned().unwrap_or_default();
            groupe::ActiveModel {
                id: Set(id),
                nom: Set(nom),
            }
            .update(&*db)
            .await
            .map(|_| ())
        })
    });

    ResourceEntry::new(meta, form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_get_fn(get_fn)
        .with_delete_fn(delete_fn)
        .with_create_fn(create_fn)
        .with_update_fn(update_fn)
}

// ─── DynForm wrappers ─────────────────────────────────────────────────────────

struct UserCreateDynWrapper(pub UserAdminCreateForm);
#[async_trait::async_trait]
impl DynForm for UserCreateDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }
    async fn save(&mut self, _db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &crate::forms::form::Forms {
        self.0.get_form()
    }
    fn get_form_mut(&mut self) -> &mut crate::forms::form::Forms {
        self.0.get_form_mut()
    }
}

struct UserEditDynWrapper(pub UserAdminEditForm);
#[async_trait::async_trait]
impl DynForm for UserEditDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }
    async fn save(&mut self, _db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &crate::forms::form::Forms {
        self.0.get_form()
    }
    fn get_form_mut(&mut self) -> &mut crate::forms::form::Forms {
        self.0.get_form_mut()
    }
}

struct DroitDynWrapper(pub DroitAdminForm);

#[async_trait::async_trait]
impl DynForm for DroitDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }
    async fn save(&mut self, _db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &crate::forms::form::Forms {
        self.0.get_form()
    }
    fn get_form_mut(&mut self) -> &mut crate::forms::form::Forms {
        self.0.get_form_mut()
    }
}

struct GroupeDynWrapper(pub GroupeAdminForm);

#[async_trait::async_trait]
impl DynForm for GroupeDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }
    async fn save(&mut self, _db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &crate::forms::form::Forms {
        self.0.get_form()
    }
    fn get_form_mut(&mut self) -> &mut crate::forms::form::Forms {
        self.0.get_form_mut()
    }
}
