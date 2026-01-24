//! Django-style ORM query builders

use sea_orm::{Condition, DatabaseConnection, DbErr, EntityTrait, ColumnTrait, QueryFilter, QuerySelect, Select};
use std::marker::PhantomData;

/// Django-style ORM manager for entities
///
/// Provides Django-like interface for querying database entities
/// Usage: `User::objects.filter(...).all(&db).await`
pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
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

    pub async fn count(&self, db: &DatabaseConnection) -> Result<u64, DbErr> {
        let items = E::find().all(db).await?;
        Ok(items.len() as u64)
    }
}

impl<E: EntityTrait> Copy for Objects<E> {}
impl<E: EntityTrait> Clone for Objects<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: EntityTrait> Default for Objects<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Django-inspired query builder for SeaORM
///
/// Wraps SeaORM's Select and provides convenient chainable methods
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    query: Select<E>,
}

impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    pub fn new(query: Select<E>) -> Self {
        Self { query }
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
}
