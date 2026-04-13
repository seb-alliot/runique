//! `Objects<E>` — Django-style manager for SeaORM entities, exposed via `impl_objects!`.

/// Django-style ORM manager for entities
///
/// This struct provides a Django-like interface for querying database entities.
/// It's designed to be used as a constant field on entity structs, enabling
/// the syntax `User::objects.filter(...)` without parentheses.
///
/// # Examples
///
/// ```rust
/// #[cfg(feature = "sqlite")]
/// async fn sqlite_objects_example() {
///     use sea_orm::entity::prelude::*;
///     use sea_orm::{Database, DbBackend, Schema, Set};
///
///     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
///     #[sea_orm(table_name = "users")]
///     pub struct Model {
///         #[sea_orm(primary_key)]
///         pub id: i32,
///         pub username: String,
///         pub age: i32,
///     }
///
///     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
///     pub enum Relation {}
///
///     impl ActiveModelBehavior for ActiveModel {}
///
///     // SQLite in-memory connection
///     let db = Database::connect("sqlite::memory:").await.unwrap();
///
///     // Table creation
///     let stmt = Schema::new(DbBackend::Sqlite).create_table_from_entity(Entity);
///     db.execute(&stmt).await.unwrap();
///
///     // Using ActiveModel for insertion
///     ActiveModel {
///         username: Set("Bob".to_owned()),
///         age: Set(25),
///         ..Default::default()
///     }
///     .insert(&db)
///     .await
///     .unwrap();
///
///     // Retrieval
///     let user: Option<Model> = Entity::find()
///         .filter(Column::Username.eq("Bob"))
///         .one(&db)
///         .await
///         .unwrap();
///     assert!(user.is_some());
/// }
///
/// #[cfg(feature = "sqlite")]
/// tokio::runtime::Runtime::new().unwrap().block_on(sqlite_objects_example());
/// ```
use super::query::RuniqueQueryBuilder;
use crate::context::template::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait};
use std::marker::PhantomData;

/// Django-style ORM manager for entities
///
#[doc = include_str!("../../../doc-tests/macro_db/model_complete.md")]
pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
}

impl<E: EntityTrait> Default for Objects<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EntityTrait> Objects<E> {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn all(&self) -> RuniqueQueryBuilder<E> {
        RuniqueQueryBuilder::new(E::find())
    }

    // In impl<E: EntityTrait> Objects<E>

    // === EXISTING (keeping current filter/exclude) ===
    pub fn filter<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RuniqueQueryBuilder::new(E::find()).filter(condition.into())
    }

    pub fn exclude<C>(&self, condition: C) -> RuniqueQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RuniqueQueryBuilder::new(E::find()).exclude(condition.into())
    }

    // === NEW : simplified vector filter ===
    pub fn filter_many<C, V, I>(&self, filters: I) -> RuniqueQueryBuilder<E>
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
        I: IntoIterator<Item = (C, V)>,
    {
        RuniqueQueryBuilder::new(E::find()).filter_many(filters)
    }

    pub fn exclude_many<C, V, I>(&self, filters: I) -> RuniqueQueryBuilder<E>
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
        I: IntoIterator<Item = (C, V)>,
    {
        RuniqueQueryBuilder::new(E::find()).exclude_many(filters)
    }

    pub async fn get(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<E::Model, DbErr> {
        E::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Record not found".to_string()))
    }

    pub async fn get_optional(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find_by_id(id).one(db).await
    }

    pub async fn count(&self, db: &DatabaseConnection) -> Result<u64, DbErr>
    where
        E::Model: Sync,
    {
        use sea_orm::PaginatorTrait;
        E::find().count(db).await
    }
    pub async fn get_or_404(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
        ctx: &Request,
        error_msg: &str,
    ) -> Result<E::Model, Response> {
        match self.get_optional(db, id).await {
            Ok(Some(entity)) => Ok(entity),
            Ok(None) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Page not found");
                context.insert("error_message", error_msg);
                match ctx.engine.tera.render("404", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Tera render 404 error: {}", e);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response())
                    }
                }
            }
            Err(_) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Server error");
                context.insert("error_message", "Database error");
                match ctx.engine.tera.render("500", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Tera render 500 error: {}", e);
                        Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response())
                    }
                }
            }
        }
    }
}

impl<E: EntityTrait> Copy for Objects<E> {}
impl<E: EntityTrait> Clone for Objects<E> {
    fn clone(&self) -> Self {
        *self
    }
}

// =====================================================
// SQLite tests enabled with "sqlite" feature
// =====================================================

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;
    use sea_orm::entity::prelude::*;

    // Test model definition
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub username: String,
        pub age: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    // Objects implementation for our test Entity
    impl Entity {
        #[allow(non_upper_case_globals)]
        pub const objects: Objects<Self> = Objects::new();
    }

    // Helper function for DB setup
    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = sea_orm::Database::connect("sqlite::memory:").await?;

        use sea_orm::Schema;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Entity);
        db.execute(&stmt).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_objects_all() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let user = ActiveModel {
            username: Set("alice".to_string()),
            age: Set(25),
            ..Default::default()
        };
        user.insert(&db).await?;

        let users = Entity::objects.all().all(&db).await?;
        assert_eq!(users.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_objects_filter() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let young = ActiveModel {
            username: Set("young".to_string()),
            age: Set(16),
            ..Default::default()
        };
        let adult = ActiveModel {
            username: Set("adult".to_string()),
            age: Set(25),
            ..Default::default()
        };
        young.insert(&db).await?;
        adult.insert(&db).await?;

        let adults = Entity::objects.filter(Column::Age.gte(18)).all(&db).await?;
        assert_eq!(adults.len(), 1);
        assert_eq!(adults[0].username, "adult");
        Ok(())
    }

    #[tokio::test]
    async fn test_objects_exclude() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let alice = ActiveModel {
            username: Set("alice".to_string()),
            age: Set(25),
            ..Default::default()
        };
        let banned = ActiveModel {
            username: Set("banned".to_string()),
            age: Set(30),
            ..Default::default()
        };
        alice.insert(&db).await?;
        banned.insert(&db).await?;

        let active_users = Entity::objects.exclude(Column::Age.eq(30)).all(&db).await?;
        assert_eq!(active_users.len(), 1);
        assert_eq!(active_users[0].username, "alice");
        Ok(())
    }

    #[tokio::test]
    async fn test_objects_count() -> Result<(), DbErr> {
        let db = setup_db().await?;

        for i in 1..=3 {
            let user = ActiveModel {
                username: Set(format!("user{}", i)),
                age: Set(20 + i),
                ..Default::default()
            };
            user.insert(&db).await?;
        }

        let count = Entity::objects.count(&db).await?;
        assert_eq!(count, 3);
        Ok(())
    }
}
