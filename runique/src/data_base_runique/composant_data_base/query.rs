use crate::data_base_runique::config::DatabaseConfig;
/// Django-inspired query builder for SeaORM
///
/// This struct wraps SeaORM's `Select<E>` and provides convenient,
/// chainable methods like `.filter()`, `.exclude()`, `.order_by_desc()`, etc.
///
/// # Examples
///
/// ```rust
/// #[cfg(feature = "sqlite")]
/// async fn sqlite_query_example() {
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
///     // Connexion SQLite en mémoire
///     let db = Database::connect("sqlite::memory:").await.unwrap();
///
///     // Création de la table
///     let stmt = Schema::new(DbBackend::Sqlite).create_table_from_entity(Entity);
///     db.execute(&stmt).await.unwrap();
///
///     // Insertion d'un utilisateur
///     ActiveModel {
///         username: Set("Alice".to_owned()),
///         age: Set(30),
///         ..Default::default()
///     }
///     .insert(&db)
///     .await
///     .unwrap();
///
///     // Vérification via query
///     let user: Option<Model> = Entity::find()
///         .filter(Column::Username.eq("Alice"))
///         .one(&db)
///         .await
///         .unwrap();
///     assert!(user.is_some());
/// }
///
/// #[cfg(feature = "sqlite")]
/// tokio::runtime::Runtime::new().unwrap().block_on(sqlite_query_example());
/// ```
use axum::response::IntoResponse;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use std::sync::Arc;

pub struct RuniqueQueryBuilder<E: EntityTrait> {
    query: Select<E>,
}

impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    pub fn new(query: Select<E>) -> Self {
        Self { query }
    }

    // Permet d'extraire la connexion depuis l'Engine directement
    pub async fn all_from_engine(
        self,
        engine: Arc<DatabaseConfig>,
    ) -> Result<Vec<E::Model>, DbErr> {
        let db = engine.connect().await?;
        self.query.all(&db).await
    }
    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<E::Model>, DbErr> {
        self.query.all(db).await
    }

    pub fn filter<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.query = self.query.filter(condition.into());
        self
    }

    pub fn exclude<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.query = self.query.filter(condition.into().not());
        self
    }

    pub fn order_by_asc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_asc(column);
        self
    }

    pub fn order_by_desc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_desc(column);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.query = self.query.limit(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.query = self.query.offset(offset);
        self
    }

    pub async fn count(self, db: &DatabaseConnection) -> Result<u64, DbErr> {
        let items = self.query.all(db).await?;
        Ok(items.len() as u64)
    }

    pub async fn first(self, db: &DatabaseConnection) -> Result<Option<E::Model>, DbErr> {
        self.query.one(db).await
    }

    pub async fn get_or_404(
        self,
        db: &DatabaseConnection,
        ctx: &crate::request_context::template_context::TemplateContext,
        error_msg: &str,
    ) -> Result<E::Model, axum::response::Response> {
        match self.first(db).await {
            Ok(Some(entity)) => Ok(entity),
            Ok(None) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Page non trouvée");
                context.insert("error_message", error_msg);

                match ctx.engine.tera.render("404", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Erreur Tera render 404: {}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "Erreur interne",
                        )
                            .into_response())
                    }
                }
            }
            Err(_) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Erreur serveur");
                context.insert("error_message", "Database error");

                match ctx.engine.tera.render("500", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Erreur Tera render 500: {}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "Erreur interne",
                        )
                            .into_response())
                    }
                }
            }
        }
    }
}

// =====================================================
// Tests SQLite activés avec feature "sqlite"
// =====================================================

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::*;
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;

    // Définition du modèle de test
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

    async fn setup_db() -> Result<DatabaseConnection, DbErr> {
        let db = sea_orm::Database::connect("sqlite::memory:").await?;

        use sea_orm::Schema;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Entity);
        db.execute(&stmt).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_querybuilder_all() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let user = ActiveModel {
            username: Set("alice".to_string()),
            age: Set(25),
            ..Default::default()
        };
        user.insert(&db).await?;

        let users = RuniqueQueryBuilder::new(Entity::find()).all(&db).await?;
        assert_eq!(users.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_filter_exclude() -> Result<(), DbErr> {
        let db = setup_db().await?;

        let alice = ActiveModel {
            username: Set("alice".to_string()),
            age: Set(25),
            ..Default::default()
        };
        let bob = ActiveModel {
            username: Set("bob".to_string()),
            age: Set(30),
            ..Default::default()
        };
        alice.insert(&db).await?;
        bob.insert(&db).await?;

        let adults = RuniqueQueryBuilder::new(Entity::find())
            .filter(Column::Age.gte(26))
            .all(&db)
            .await?;
        assert_eq!(adults.len(), 1);
        assert_eq!(adults[0].username, "bob");

        let not_bob = RuniqueQueryBuilder::new(Entity::find())
            .exclude(Column::Username.eq("bob"))
            .all(&db)
            .await?;
        assert_eq!(not_bob.len(), 1);
        assert_eq!(not_bob[0].username, "alice");

        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_order_limit_count_first() -> Result<(), DbErr> {
        let db = setup_db().await?;

        for i in 1..=3 {
            let user = ActiveModel {
                username: Set(format!("user{}", i)),
                age: Set(20 + i),
                ..Default::default()
            };
            user.insert(&db).await?;
        }

        let count = RuniqueQueryBuilder::new(Entity::find()).count(&db).await?;
        assert_eq!(count, 3);

        let first = RuniqueQueryBuilder::new(Entity::find())
            .order_by_asc(Column::Age)
            .first(&db)
            .await?
            .unwrap();
        assert_eq!(first.age, 21);

        let limited = RuniqueQueryBuilder::new(Entity::find())
            .limit(2)
            .all(&db)
            .await?;
        assert_eq!(limited.len(), 2);

        Ok(())
    }
}
