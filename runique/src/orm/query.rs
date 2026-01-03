use crate::processor::Template;
use axum::response::Response;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
/// Wrapper pour Select avec méthodes pratiques et chainables
///
/// Cette struct encapsule un `Select<E>` de SeaORM et fournit
/// des méthodes pratiques inspirées de Django ORM.
pub struct RuniqueQueryBuilder<E: EntityTrait> {
    select: Select<E>,
}

impl<E: EntityTrait> RuniqueQueryBuilder<E> {
    /// Crée un nouveau QueryBuilder à partir d'un Select
    pub fn new(select: Select<E>) -> Self {
        Self { select }
    }

    /// Récupère tous les enregistrements
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let users = User::objects.all().all(&db).await?;
    /// ```
    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<E::Model>, DbErr> {
        self.select.all(db).await
    }

    /// Filtre par une condition
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let adults = User::objects
    ///     .filter(user::Column::Age.gte(18))
    ///     .all(&db)
    ///     .await?;
    /// ```
    /// Filtre par une condition
    pub fn filter<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.select = self.select.filter(condition.into());
        self
    }

    /// Exclut les enregistrements correspondant à une condition
    pub fn exclude<C>(mut self, condition: C) -> Self
    where
        C: Into<Condition>,
    {
        self.select = self.select.filter(condition.into().not());
        self
    }

    /// Ordre croissant par colonne
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let users = User::objects
    ///     .order_by_asc(user::Column::Username)
    ///     .all(&db)
    ///     .await?;
    /// ```
    pub fn order_by_asc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.select = self.select.order_by_asc(column);
        self
    }

    /// Ordre décroissant par colonne
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let users = User::objects
    ///     .order_by_desc(user::Column::CreatedAt)
    ///     .all(&db)
    ///     .await?;
    /// ```
    pub fn order_by_desc<C: ColumnTrait>(mut self, column: C) -> Self {
        self.select = self.select.order_by_desc(column);
        self
    }

    /// Limite le nombre de résultats
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let users = User::objects
    ///     .limit(10)
    ///     .all(&db)
    ///     .await?;
    /// ```
    pub fn limit(mut self, limit: u64) -> Self {
        self.select = self.select.limit(limit);
        self
    }

    /// Skip les n premiers résultats
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let page_2 = User::objects
    ///     .offset(20)
    ///     .limit(10)
    ///     .all(&db)
    ///     .await?;
    /// ```
    pub fn offset(mut self, offset: u64) -> Self {
        self.select = self.select.offset(offset);
        self
    }

    /// Compte le nombre d'enregistrements
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let count = User::objects
    ///     .filter(user::Column::Age.gte(18))
    ///     .count(&db)
    ///     .await?;
    /// ```
    pub async fn count(self, db: &DatabaseConnection) -> Result<u64, DbErr> {
        let items = self.select.all(db).await?;
        Ok(items.len() as u64)
    }

    /// Récupère le premier résultat
    ///
    /// # Exemple
    /// ```rust,ignore
    /// if let Some(user) = User::objects
    ///     .filter(user::Column::Username.eq("alice"))
    ///     .first(&db)
    ///     .await? {
    ///     println!("Found: {}", user.username);
    /// }
    /// ```
    pub async fn first(self, db: &DatabaseConnection) -> Result<Option<E::Model>, DbErr> {
        self.select.one(db).await
    }

    /// Récupère le premier résultat ou retourne une 404
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let user = User::objects
    ///     .filter(user::Column::Username.eq("alice"))
    ///     .get_or_404(&db, &template, "Utilisateur introuvable")
    ///     .await?;
    /// ```
    pub async fn get_or_404(
        self,
        db: &DatabaseConnection,
        template: &Template,
        error_msg: &str,
    ) -> Result<E::Model, Response> {
        match self.first(db).await {
            Ok(Some(entity)) => Ok(entity),
            Ok(None) => Err(template.render_404(error_msg)),
            Err(_) => Err(template.render_500("Erreur de base de données")),
        }
    }
}
