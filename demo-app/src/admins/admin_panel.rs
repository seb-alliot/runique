// AUTO-admin_panel — DO NOT EDIT MANUALLY
// admin_panel by `runique start` from src/admin.rs

use runique::prelude::*;

use crate::formulaire::RegisterForm;
use crate::entities::users;

// ── DynForm wrapper pour RegisterForm ──
struct RegisterFormDynWrapper(pub RegisterForm);

#[async_trait]
impl DynForm for RegisterFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await.map(|_model| ())
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

/// Construit le registre admin au boot.
/// Appelé par le builder de l'application.
pub fn admin_register() -> AdminRegistry {
    let mut registry = AdminRegistry::new();

    // ── Ressource : users ──
    let meta = AdminResource::new(
        "users",
        "crate::entities::users::Model",
        "RegisterForm",
        "Utilisateurs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = RegisterForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(RegisterFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = users::Entity::find().all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: i32| {
        Box::pin(async move {
            let row = users::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: i32| {
        Box::pin(async move {
            users::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            users::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: i32, data: StrMap| {
        Box::pin(async move {
            users::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
    );

    registry
}

/// Construit le Router axum du prototype admin pour le préfixe donné.
/// À passer à `.with_admin(|a| a.routes(admins::routes("/admin")))` dans main.rs.
pub fn routes(prefix: &str) -> runique::axum::Router {
    use runique::axum::{routing::get, Extension};
    let p = prefix.trim_end_matches('/');
    let config = Arc::new(AdminConfig::new().prefix(prefix));
    let state = Arc::new(PrototypeAdminState {
        registry: Arc::new(admin_register()),
        config: config.clone(),
    });
    runique::axum::Router::new()
        .route(&format!("{}/{{resource}}/{{action}}", p), get(admin_get).post(admin_post))
        .route(&format!("{}/{{resource}}/{{id}}/{{action}}", p), get(admin_get_id).post(admin_post_id))
        .layer(Extension(state))
}
