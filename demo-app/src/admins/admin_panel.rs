// AUTO-admin_panel — DO NOT EDIT MANUALLY
// admin_panel by `runique start` from src/admin.rs

use runique::prelude::*;

use crate::entities::blog;
use crate::entities::changelog_entry;
use crate::entities::code_example;
use crate::entities::demo_category;
use crate::entities::demo_page;
use crate::entities::demo_section;
use crate::entities::form_field;
use crate::entities::known_issue;
use crate::entities::page_doc_link;
use crate::entities::roadmap_entry;
use crate::entities::users;

// ── DynForm wrapper pour users::AdminForm ──
struct UsersAdminFormDynWrapper(pub users::AdminForm);

#[async_trait]
impl DynForm for UsersAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour blog::AdminForm ──
struct BlogAdminFormDynWrapper(pub blog::AdminForm);

#[async_trait]
impl DynForm for BlogAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour changelog_entry::AdminForm ──
struct ChangelogEntryAdminFormDynWrapper(pub changelog_entry::AdminForm);

#[async_trait]
impl DynForm for ChangelogEntryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour roadmap_entry::AdminForm ──
struct RoadmapEntryAdminFormDynWrapper(pub roadmap_entry::AdminForm);

#[async_trait]
impl DynForm for RoadmapEntryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour known_issue::AdminForm ──
struct KnownIssueAdminFormDynWrapper(pub known_issue::AdminForm);

#[async_trait]
impl DynForm for KnownIssueAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_category::AdminForm ──
struct DemoCategoryAdminFormDynWrapper(pub demo_category::AdminForm);

#[async_trait]
impl DynForm for DemoCategoryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_page::AdminForm ──
struct DemoPageAdminFormDynWrapper(pub demo_page::AdminForm);

#[async_trait]
impl DynForm for DemoPageAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_section::AdminForm ──
struct DemoSectionAdminFormDynWrapper(pub demo_section::AdminForm);

#[async_trait]
impl DynForm for DemoSectionAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour code_example::AdminForm ──
struct CodeExampleAdminFormDynWrapper(pub code_example::AdminForm);

#[async_trait]
impl DynForm for CodeExampleAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour page_doc_link::AdminForm ──
struct PageDocLinkAdminFormDynWrapper(pub page_doc_link::AdminForm);

#[async_trait]
impl DynForm for PageDocLinkAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour form_field::AdminForm ──
struct FormFieldAdminFormDynWrapper(pub form_field::AdminForm);

#[async_trait]
impl DynForm for FormFieldAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
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
        "AdminForm",
        "Utilisateurs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = users::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(UsersAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = users::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { users::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = users::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            users::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : blog ──
    let meta = AdminResource::new(
        "blog",
        "crate::entities::blog::Model",
        "AdminForm",
        "Articles",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = blog::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(BlogAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = blog::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { blog::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = blog::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            blog::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : changelog_entry ──
    let meta = AdminResource::new(
        "changelog_entry",
        "crate::entities::changelog_entry::Model",
        "AdminForm",
        "Changelog",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    changelog_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(ChangelogEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = changelog_entry::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb| {
        Box::pin(async move { changelog_entry::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = changelog_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            changelog_entry::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : roadmap_entry ──
    let meta = AdminResource::new(
        "roadmap_entry",
        "crate::entities::roadmap_entry::Model",
        "AdminForm",
        "Roadmap",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    roadmap_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(RoadmapEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = roadmap_entry::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb| {
        Box::pin(async move { roadmap_entry::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = roadmap_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            roadmap_entry::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : known_issue ──
    let meta = AdminResource::new(
        "known_issue",
        "crate::entities::known_issue::Model",
        "AdminForm",
        "Problèmes connus",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    known_issue::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(KnownIssueAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = known_issue::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { known_issue::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = known_issue::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            known_issue::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_category ──
    let meta = AdminResource::new(
        "demo_category",
        "crate::entities::demo_category::Model",
        "AdminForm",
        "Catégories",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    demo_category::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoCategoryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = demo_category::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb| {
        Box::pin(async move { demo_category::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_category::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_category::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_page ──
    let meta = AdminResource::new(
        "demo_page",
        "crate::entities::demo_page::Model",
        "AdminForm",
        "Pages",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = demo_page::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoPageAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = demo_page::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { demo_page::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_page::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_page::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_section ──
    let meta = AdminResource::new(
        "demo_section",
        "crate::entities::demo_section::Model",
        "AdminForm",
        "Sections",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    demo_section::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoSectionAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = demo_section::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { demo_section::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_section::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_section::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : code_example ──
    let meta = AdminResource::new(
        "code_example",
        "crate::entities::code_example::Model",
        "AdminForm",
        "Exemples de code",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    code_example::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(CodeExampleAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = code_example::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { code_example::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = code_example::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            code_example::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : page_doc_link ──
    let meta = AdminResource::new(
        "page_doc_link",
        "crate::entities::page_doc_link::Model",
        "AdminForm",
        "Liens documentation",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    page_doc_link::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(PageDocLinkAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = page_doc_link::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb| {
        Box::pin(async move { page_doc_link::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = page_doc_link::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            page_doc_link::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : form_field ──
    let meta = AdminResource::new(
        "form_field",
        "crate::entities::form_field::Model",
        "AdminForm",
        "Champs formulaire",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = form_field::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(FormFieldAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb| {
        Box::pin(async move {
            let rows = form_field::Entity::find().all(&*db).await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn =
        Arc::new(|db: ADb| Box::pin(async move { form_field::Entity::find().count(&*db).await }));

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = form_field::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            form_field::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    registry
}

/// Construit le Router axum du prototype admin pour le préfixe donné.
/// À passer à `.with_admin(|a| a.routes(admins::routes("/admin")))` dans main.rs.
pub fn routes(prefix: &str) -> runique::axum::Router {
    let p = prefix.trim_end_matches('/');
    let config = Arc::new(AdminConfig::new().prefix(prefix));
    let state = Arc::new(PrototypeAdminState {
        registry: Arc::new(admin_register()),
        config: config.clone(),
    });
    runique::axum::Router::new()
        .route(
            &format!("{}/{{resource}}/{{action}}", p),
            get(admin_get).post(admin_post),
        )
        .route(
            &format!("{}/{{resource}}/{{id}}/{{action}}", p),
            get(admin_get_id).post(admin_post_id),
        )
        .layer(Extension(state))
}

/// Retourne l'état partagé du prototype admin (pour le dashboard).
pub fn admin_state() -> std::sync::Arc<PrototypeAdminState> {
    let config = Arc::new(AdminConfig::new());
    std::sync::Arc::new(PrototypeAdminState {
        registry: Arc::new(admin_register()),
        config,
    })
}
