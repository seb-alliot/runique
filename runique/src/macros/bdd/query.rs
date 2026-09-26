//! `RuniqueQueryBuilder<E>` — Django-style query builder: filter, exclude, order_by, limit, all, get, count…
use crate::db::DatabaseConfig;
/// Django-inspired query builder for SeaORM
///
/// This struct wraps SeaORM's `Select<E>` and provides convenient,
/// chainable methods like `.filter()`, `.exclude()`, `.order_by_desc()`, etc.
///
/// # Examples
///
/// ```rust,ignore
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
///     // SQLite in-memory connection
///     let db = Database::connect("sqlite::memory:").await.unwrap();
///
///     // Table creation
///     let stmt = Schema::new(DbBackend::Sqlite).create_table_from_entity(Entity);
///     db.execute(&stmt).await.unwrap();
///
///     // User insertion
///     ActiveModel {
///         username: Set("Alice".to_owned()),
///         age: Set(30),
///         ..Default::default()
///     }
///     .insert(&db)
///     .await
///     .unwrap();
///
///     // Verification via query
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
use crate::utils::aliases::ADb;
use axum::response::IntoResponse;
use sea_orm::{
    ColumnTrait, Condition, DbErr, EntityTrait, ExprTrait, JoinType, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use std::sync::Arc;

/// Django-style query builder wrapping a SeaORM `Select<E>`. Built by
/// [`Queryable::objects`] or the `search!{}` macro, and consumed by a
/// terminal method (`.all()`, `.one()`, `.first()`, `.count()`, …).
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    query: Select<E>,
}

impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    /// Wraps an existing SeaORM `Select<E>` query.
    pub fn new(query: Select<E>) -> Self {
        Self { query }
    }

    /// Runs the query using a connection obtained straight from `engine`,
    /// for call sites that only have a `DatabaseConfig` and not an open
    /// `DatabaseConnection`.
    pub async fn all_from_engine(
        self,
        engine: Arc<DatabaseConfig>,
    ) -> Result<Vec<E::Model>, DbErr> {
        let db = engine.connect().await?;
        self.query.all(&db).await
    }
    /// Executes the query and returns every matching row.
    pub async fn all(self, db: &ADb) -> Result<Vec<E::Model>, DbErr> {
        self.query.all(db).await
    }

    // In impl<E: EntityTrait> RuniqueQueryBuilder<E>

    // === EXISTING (keeping current filter/exclude) ===
    /// Adds a `WHERE` condition, ANDed with any condition already on the query.
    pub fn filter<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.query = self.query.filter(condition.into());
        self
    }

    /// Adds the negation of `condition` — rows matching it are excluded from the result.
    pub fn exclude<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.query = self.query.filter(condition.into().not());
        self
    }

    // === NEW : vector version, simplified syntax ===
    /// Shorthand for filtering on several `(column, value)` equality pairs at once —
    /// equivalent to calling `.filter(col.eq(val))` for each pair (conditions are ANDed).
    pub fn filter_many<C, V, I>(mut self, filters: I) -> Self
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
        I: IntoIterator<Item = (C, V)>,
    {
        for (col, val) in filters {
            self.query = self.query.filter(col.eq(val));
        }
        self
    }

    /// Shorthand for excluding rows on several `(column, value)` pairs at once —
    /// equivalent to calling `.exclude(col.eq(val))` for each pair (conditions are ANDed,
    /// so a row must differ from every pair's value to be kept).
    pub fn exclude_many<C, V, I>(mut self, filters: I) -> Self
    where
        C: ColumnTrait,
        V: Into<sea_orm::Value>,
        I: IntoIterator<Item = (C, V)>,
    {
        for (col, val) in filters {
            self.query = self.query.filter(col.eq(val).not());
        }
        self
    }

    /// Orders results by `column` ascending.
    pub fn order_by_asc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_asc(column);
        self
    }

    /// Orders results by `column` descending.
    pub fn order_by_desc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_desc(column);
        self
    }

    /// Alias for [`order_by_asc`](Self::order_by_asc), used by the `search!{}` macro's
    /// `asc Column` syntax.
    pub fn asc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_asc(column);
        self
    }

    /// Alias for [`order_by_desc`](Self::order_by_desc), used by the `search!{}` macro's
    /// `desc Column` syntax.
    pub fn desc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.query = self.query.order_by_desc(column);
        self
    }

    /// Orders results randomly. `RANDOM()` (SQLite/Postgres) vs `RAND()`
    /// (MySQL/MariaDB) is picked from `db.get_database_backend()`, following
    /// the same per-backend-fragment pattern as `admin::helper::sql_dialect`.
    pub fn order_by_random(mut self, db: &ADb) -> Self {
        use sea_orm::Order;
        use sea_query::Expr;
        let func = match db.get_database_backend() {
            sea_orm::DbBackend::MySql => "RAND()",
            _ => "RANDOM()",
        };
        self.query = self.query.order_by(Expr::cust(func), Order::Asc);
        self
    }

    /// Orders results by an arbitrary SeaORM expression and direction, for
    /// cases the `asc`/`desc` helpers can't express directly.
    pub fn order_by_expr<T>(mut self, expr: T, order: sea_orm::Order) -> Self
    where
        T: sea_orm::IntoSimpleExpr,
    {
        self.query = self.query.order_by(expr, order);
        self
    }

    /// Consumes the builder and returns the underlying SeaORM `Select<E>`,
    /// to drop down to raw SeaORM query methods not exposed here (e.g.
    /// `.select_only()`, `.column()`, `.into_tuple()`).
    pub fn into_select(self) -> Select<E> {
        self.query
    }

    /// Caps the number of rows the query can return.
    pub fn limit(mut self, limit: u64) -> Self {
        self.query = self.query.limit(limit);
        self
    }

    /// Skips the first `offset` rows before returning results.
    pub fn offset(mut self, offset: u64) -> Self {
        self.query = self.query.offset(offset);
        self
    }

    /// Executes the query and returns the number of matching rows.
    pub async fn count(self, db: &ADb) -> Result<u64, DbErr>
    where
        E::Model: Sync,
    {
        use sea_orm::PaginatorTrait;
        self.query.count(db).await
    }

    /// Executes the query and returns the first matching row, or `None` if
    /// there are no matches. Unlike [`one`](Self::one), this does not check
    /// whether more than one row would match.
    pub async fn first(self, db: &ADb) -> Result<Option<E::Model>, DbErr> {
        self.query.one(db).await
    }

    /// Executes the query expecting exactly one match: fetches up to two
    /// rows and returns `Err` if both come back, so callers can rely on the
    /// result being unique instead of silently taking the first row.
    pub async fn one(self, db: &ADb) -> Result<Option<E::Model>, DbErr>
    where
        E::Model: Sync,
    {
        use sea_orm::PaginatorTrait;
        let mut results = self.query.paginate(db, 2).fetch_page(0).await?;
        match results.len() {
            0 => Ok(None),
            1 => Ok(Some(results.remove(0))),
            _ => Err(DbErr::Custom(
                "search!.one(): multiple rows returned".to_string(),
            )),
        }
    }

    /// Adds an inner join on `rel`.
    pub fn join(mut self, rel: sea_orm::RelationDef) -> Self {
        self.query = self.query.join(JoinType::InnerJoin, rel);
        self
    }

    /// Adds a left join on `rel`.
    pub fn left_join(mut self, rel: sea_orm::RelationDef) -> Self {
        self.query = self.query.join(JoinType::LeftJoin, rel);
        self
    }

    /// Loads the related entity at the same time — returns `Vec<(E::Model, Option<R::Model>)>`.
    ///
    /// ```rust,ignore
    /// search!(ContributionEntity => desc Id,)
    ///     .also_related(eihwaz_users::Entity)
    ///     .all(db).await
    /// ```
    pub fn also_related<R>(self, r: R) -> sea_orm::SelectTwo<E, R>
    where
        R: sea_orm::EntityTrait,
        E: sea_orm::Related<R>,
    {
        self.query.find_also_related(r)
    }

    pub async fn get_or_404(
        self,
        db: &ADb,
        ctx: &crate::context::template::Request,
        error_msg: &str,
    ) -> Result<E::Model, axum::response::Response> {
        match self.first(db).await {
            Ok(Some(entity)) => Ok(entity),
            Ok(None) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Page not found");
                context.insert("error_message", error_msg);

                match ctx.engine.tera.render("404.html", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Tera render 404 error: {}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal error",
                        )
                            .into_response())
                    }
                }
            }
            Err(_) => {
                let mut context = ctx.context.clone();
                context.insert("title", "Server error");
                context.insert("error_message", "Database error");

                match ctx.engine.tera.render("500.html", &context) {
                    Ok(html) => Err(axum::response::Html(html).into_response()),
                    Err(e) => {
                        tracing::error!("Tera render 500 error: {}", e);
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal error",
                        )
                            .into_response())
                    }
                }
            }
        }
    }
}

pub trait Queryable {
    fn objects() -> RuniqueQueryBuilder<Self>
    where
        Self: Sized + EntityTrait,
    {
        RuniqueQueryBuilder::new(Self::find())
    }
}

impl<T: EntityTrait> Queryable for T {}

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

    async fn setup_db() -> Result<ADb, DbErr> {
        let db = sea_orm::Database::connect("sqlite::memory:").await?;

        use sea_orm::Schema;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Entity);
        db.execute(&stmt).await?;

        Ok(ADb::from_connection(db))
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

    #[tokio::test]
    async fn test_querybuilder_filter_many() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for (name, age) in [("alice", 25), ("bob", 30), ("carol", 25)] {
            ActiveModel {
                username: Set(name.to_string()),
                age: Set(age),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
        let result = RuniqueQueryBuilder::new(Entity::find())
            .filter_many([(Column::Age, 25)])
            .all(&db)
            .await?;
        assert_eq!(result.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_exclude_many() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for (name, age) in [("alice", 25), ("bob", 30)] {
            ActiveModel {
                username: Set(name.to_string()),
                age: Set(age),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
        let result = RuniqueQueryBuilder::new(Entity::find())
            .exclude_many([(Column::Username, "bob".to_string())])
            .all(&db)
            .await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].username, "alice");
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_order_by_desc() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for i in 1..=3 {
            ActiveModel {
                username: Set(format!("u{}", i)),
                age: Set(20 + i),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
        let result = RuniqueQueryBuilder::new(Entity::find())
            .order_by_desc(Column::Age)
            .first(&db)
            .await?
            .unwrap();
        assert_eq!(result.age, 23);
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_desc_asc_aliases() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for i in 1..=3 {
            ActiveModel {
                username: Set(format!("u{}", i)),
                age: Set(20 + i),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
        let asc_first = RuniqueQueryBuilder::new(Entity::find())
            .asc(Column::Age)
            .first(&db)
            .await?
            .unwrap();
        assert_eq!(asc_first.age, 21);

        let desc_first = RuniqueQueryBuilder::new(Entity::find())
            .desc(Column::Age)
            .first(&db)
            .await?
            .unwrap();
        assert_eq!(desc_first.age, 23);
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_offset() -> Result<(), DbErr> {
        let db = setup_db().await?;
        for i in 1..=4 {
            ActiveModel {
                username: Set(format!("u{}", i)),
                age: Set(i),
                ..Default::default()
            }
            .insert(&db)
            .await?;
        }
        let result = RuniqueQueryBuilder::new(Entity::find())
            .order_by_asc(Column::Age)
            .limit(10)
            .offset(2)
            .all(&db)
            .await?;
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].age, 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_querybuilder_into_select() -> Result<(), DbErr> {
        let db = setup_db().await?;
        ActiveModel {
            username: Set("alice".to_string()),
            age: Set(25),
            ..Default::default()
        }
        .insert(&db)
        .await?;
        let select = RuniqueQueryBuilder::new(Entity::find()).into_select();
        let result = select.all(&db).await?;
        assert_eq!(result.len(), 1);
        Ok(())
    }
}
