use std::sync::Arc;

use axum::http::Method;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QuerySelect};

use crate::admin::{
    forms::{UserAdminCreateForm, UserAdminEditForm},
    helper::{
        dyn_form::DynForm,
        resource_entry::{
            CountFn, CreateFn, DeleteFn, FormBuilder, GetFn, ListFn, ListParams, ResourceEntry,
            SortDir, UpdateFn,
        },
    },
    resource::AdminResource,
};
use crate::forms::field::RuniqueForm;
use crate::utils::{
    aliases::{ADb, ATera, StrMap},
    constante::{
        admin_context::permission::GROUPES,
        session_key::session::{
            IS_ACTIVE, SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY,
        },
    },
    forms::parse_bool,
    trad::{t, tf},
};

pub(super) fn user_entry() -> ResourceEntry {
    use crate::auth::user;
    use crate::utils::pk::Pk;

    let meta = AdminResource::new(
        "users",
        "runique::auth::user::Model",
        "UserAdminCreateForm",
        "Users",
        vec!["admin".to_string()],
    )
    .inject_password(true);

    let form_builder: FormBuilder = Arc::new(
        |db: ADb,
         _vec: Vec<std::string::String>,
         data: StrMap,
         tera: ATera,
         csrf: String,
         method: Method| {
            Box::pin(async move {
                let mut form =
                    UserAdminCreateForm::build_with_data(&data, tera, &csrf, method).await;

                let groupes = crate::admin::permissions::groupe::Entity::find()
                    .all(&*db)
                    .await
                    .unwrap_or_default();
                let choices = groupes
                    .into_iter()
                    .map(|g| crate::forms::fields::ChoiceOption::new(&g.id.to_string(), &g.nom))
                    .collect::<Vec<_>>();
                form.get_form_mut().fields.insert(
                    GROUPES.to_string(),
                    Box::new(
                        crate::forms::fields::CheckboxField::new(GROUPES)
                            .choices(choices)
                            .label(t("admin.groups").as_ref()),
                    ),
                );

                Box::new(UserCreateDynWrapper(form)) as Box<dyn DynForm>
            })
        },
    );

    let edit_form_builder: FormBuilder = Arc::new(
        |db: ADb,
         _vec: Vec<std::string::String>,
         data: StrMap,
         tera: ATera,
         csrf: String,
         method: Method| {
            Box::pin(async move {
                let mut form = UserAdminEditForm::build_with_data(&data, tera, &csrf, method).await;

                let groupes = crate::admin::permissions::groupe::Entity::find()
                    .all(&*db)
                    .await
                    .unwrap_or_default();
                let selected = data.get(GROUPES).cloned().unwrap_or_default();
                let choices = groupes
                    .into_iter()
                    .map(|g| {
                        let id_str = g.id.to_string();
                        let mut opt = crate::forms::fields::ChoiceOption::new(&id_str, &g.nom);
                        if selected.split(',').any(|s| s.trim() == id_str) {
                            opt = opt.selected();
                        }
                        opt
                    })
                    .collect::<Vec<_>>();

                let mut field = crate::forms::fields::CheckboxField::new(GROUPES)
                    .choices(choices)
                    .label(t("admin.groups").as_ref());
                {
                    use crate::forms::base::FormField;
                    field.set_value(&selected);
                }

                form.get_form_mut()
                    .fields
                    .insert(GROUPES.to_string(), Box::new(field));

                Box::new(UserEditDynWrapper(form)) as Box<dyn DynForm>
            })
        },
    );

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, ExprTrait, Func, Order},
            };
            let mut query = user::Entity::find();
            if let Some(ref col) = params.sort_by
                && !col.is_empty()
                && col.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                // Identifier guard (blocks injection on every backend) + bound value.
                if col.is_empty() || !col.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                    continue;
                }
                query = query.filter(
                    Expr::col(Alias::new(col.as_str()))
                        .cast_as(Alias::new("TEXT"))
                        .eq(val.clone()),
                );
            }
            if let Some(ref search_str) = params.search {
                let pattern = format!("%{}%", search_str.to_lowercase());
                let mut search_cond = sea_orm::Condition::any();
                let cols = vec!["id", "username", "email"];
                for col in cols {
                    search_cond = search_cond.add(
                        Expr::expr(Func::lower(
                            Expr::col(Alias::new(col)).cast_as(Alias::new("TEXT")),
                        ))
                        .like(pattern.clone()),
                    );
                }
                query = query.filter(search_cond);
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, ExprTrait, Func},
            };
            let mut query = user::Entity::find();
            if let Some(ref search_str) = _search {
                let pattern = format!("%{}%", search_str.to_lowercase());
                let mut search_cond = sea_orm::Condition::any();
                let cols = vec!["id", "username", "email"];
                for col in cols {
                    search_cond = search_cond.add(
                        Expr::expr(Func::lower(
                            Expr::col(Alias::new(col)).cast_as(Alias::new("TEXT")),
                        ))
                        .like(pattern.clone()),
                    );
                }
                query = query.filter(search_cond);
            }
            query.count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            use crate::admin::permissions::users_groupes;
            use sea_orm::{ColumnTrait, QueryFilter};

            let id = id
                .parse::<Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            let row = user::Entity::find_by_id(id).one(&*db).await?;
            let Some(row) = row else { return Ok(None) };

            let mut value = serde_json::to_value(&row).unwrap_or(serde_json::Value::Null);

            let liens: Vec<users_groupes::Model> = users_groupes::Entity::find()
                .filter(users_groupes::Column::UserId.eq(id))
                .all(&*db)
                .await
                .unwrap_or_default();
            let groupes_str = liens
                .iter()
                .map(|l| l.groupe_id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            if let serde_json::Value::Object(ref mut map) = value {
                map.insert(GROUPES.to_string(), serde_json::Value::String(groupes_str));
            }

            Ok(Some(value))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            user::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            use crate::admin::permissions::users_groupes;

            let now = Some(chrono::Utc::now().naive_utc());
            let username = data.get("username").cloned().unwrap_or_default();
            let email = data.get("email").cloned().unwrap_or_default();
            let inserted = user::ActiveModel {
                username: Set(username.clone()),
                email: Set(email.clone()),
                password: Set(data.get("password").cloned().unwrap_or_default()),
                is_active: Set(parse_bool(&data, IS_ACTIVE)),
                is_staff: Set(parse_bool(&data, SESSION_USER_IS_STAFF_KEY)),
                is_superuser: Set(parse_bool(&data, SESSION_USER_IS_SUPERUSER_KEY)),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&*db)
            .await
            .map_err(|e| {
                if super::is_unique_violation(&e) {
                    let msg = e.to_string().to_lowercase();
                    if msg.contains("username") {
                        sea_orm::DbErr::Custom(tf(
                            "admin.builtin.user_username_exists",
                            &[&username],
                        ))
                    } else if msg.contains("email") {
                        sea_orm::DbErr::Custom(tf("admin.builtin.user_email_exists", &[&email]))
                    } else {
                        e
                    }
                } else {
                    e
                }
            })?;

            if let Some(groupes_str) = data.get(GROUPES) {
                for id_str in groupes_str.split(',') {
                    let id_str = id_str.trim();
                    if id_str.is_empty() {
                        continue;
                    }
                    if let Ok(groupe_id) = id_str.parse::<crate::utils::pk::Pk>()
                        && let Err(e) = (users_groupes::ActiveModel {
                            user_id: Set(inserted.id),
                            groupe_id: Set(groupe_id),
                        }
                        .insert(&*db)
                        .await)
                    {
                        tracing::warn!(user_id = %inserted.id, groupe_id = %groupe_id, error = %e, "user→group assignment failed (create)");
                    }
                }
            }

            Ok(())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            use crate::admin::permissions::users_groupes;
            use sea_orm::ColumnTrait;

            let id = id
                .parse::<Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            user::ActiveModel {
                id: Set(id),
                username: Set(data.get("username").cloned().unwrap_or_default()),
                email: Set(data.get("email").cloned().unwrap_or_default()),
                is_active: Set(parse_bool(&data, IS_ACTIVE)),
                is_staff: Set(parse_bool(&data, SESSION_USER_IS_STAFF_KEY)),
                is_superuser: Set(parse_bool(&data, SESSION_USER_IS_SUPERUSER_KEY)),
                updated_at: Set(Some(chrono::Utc::now().naive_utc())),
                ..Default::default()
            }
            .update(&*db)
            .await?;

            {
                use sea_orm::QueryFilter;
                users_groupes::Entity::delete_many()
                    .filter(users_groupes::Column::UserId.eq(id))
                    .exec(&*db)
                    .await?;
            }

            if let Some(groupes_str) = data.get(GROUPES) {
                for id_str in groupes_str.split(',') {
                    let id_str = id_str.trim();
                    if id_str.is_empty() {
                        continue;
                    }
                    if let Ok(groupe_id) = id_str.parse::<crate::utils::pk::Pk>()
                        && let Err(e) = (users_groupes::ActiveModel {
                            user_id: Set(id),
                            groupe_id: Set(groupe_id),
                        }
                        .insert(&*db)
                        .await)
                    {
                        tracing::warn!(user_id = %id, groupe_id = %groupe_id, error = %e, "user→group assignment failed (update)");
                    }
                }
            }

            Ok(())
        })
    });

    let partial_update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            let mut model = user::ActiveModel {
                id: ::sea_orm::ActiveValue::Unchanged(id),
                updated_at: Set(Some(chrono::Utc::now().naive_utc())),
                ..Default::default()
            };
            if data.contains_key(IS_ACTIVE) {
                model.is_active = Set(parse_bool(&data, IS_ACTIVE));
            }
            if data.contains_key(SESSION_USER_IS_STAFF_KEY) {
                model.is_staff = Set(parse_bool(&data, SESSION_USER_IS_STAFF_KEY));
            }
            if data.contains_key(SESSION_USER_IS_SUPERUSER_KEY) {
                model.is_superuser = Set(parse_bool(&data, SESSION_USER_IS_SUPERUSER_KEY));
            }
            if let Some(v) = data.get("username").filter(|v| !v.is_empty()) {
                model.username = Set(v.clone());
            }
            if let Some(v) = data.get("email").filter(|v| !v.is_empty()) {
                model.email = Set(v.clone());
            }
            model.update(&*db).await?;
            Ok(())
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
        .with_partial_update_fn(partial_update_fn)
}

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
