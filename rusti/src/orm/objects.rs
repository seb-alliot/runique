use sea_orm::{EntityTrait, Condition, DatabaseConnection, DbErr};
use std::marker::PhantomData;
use super::query::RustiQueryBuilder;
use crate::processor::processor::Template;
use axum::response::Response;

/// Struct qui encapsule la logique "objects"
///
/// Cette struct permet d'avoir la syntaxe `User::objects.filter(...)`
/// sans parenthèses après `objects`, exactement comme Django.
pub struct Objects<E: EntityTrait> {
    _phantom: PhantomData<E>,
}

impl<E: EntityTrait> Objects<E> {
    /// Constructeur const pour pouvoir l'utiliser en constante
    ///
    /// Ceci permet de faire : `pub const objects: Objects<Self> = Objects::new();`
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Retourne un QueryBuilder pour tous les enregistrements
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let users = User::objects.all().all(&db).await?;
    /// ```
    pub fn all(&self) -> RustiQueryBuilder<E> {
        RustiQueryBuilder::new(E::find())
    }

    /// Filtre les enregistrements selon une condition
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let adults = User::objects
    ///     .filter(user::Column::Age.gte(18))
    ///     .all(&db)
    ///     .await?;
    /// ```
    /// Filtre les enregistrements selon une condition
    pub fn filter<C>(&self, condition: C) -> RustiQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RustiQueryBuilder::new(E::find()).filter(condition.into())
    }

    /// Exclut des enregistrements
    pub fn exclude<C>(&self, condition: C) -> RustiQueryBuilder<E>
    where
        C: Into<Condition>,
    {
        RustiQueryBuilder::new(E::find()).exclude(condition.into())
    }

    /// Récupère un enregistrement par ID (erreur si non trouvé)
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let user = User::objects.get(&db, 1).await?;
    /// ```
    pub async fn get(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<E::Model, DbErr> {
        E::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(
                "Record not found".to_string()
            ))
    }

    /// Récupère un enregistrement par ID (None si non trouvé)
    ///
    /// # Exemple
    /// ```rust,ignore
    /// if let Some(user) = User::objects.get_optional(&db, 999).await? {
    ///     println!("Found: {}", user.username);
    /// }
    /// ```
    pub async fn get_optional(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find_by_id(id).one(db).await
    }

    /// Compte tous les enregistrements
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let count = User::objects.count(&db).await?;
    /// ```
    pub async fn count(&self, db: &DatabaseConnection) -> Result<u64, DbErr> {
        let items = E::find().all(db).await?;
        Ok(items.len() as u64)
    }

    /// Récupère un enregistrement ou retourne une erreur 404
    ///
    /// # Exemple
    /// ```rust,ignore
    /// let user = User::objects
    ///     .get_or_404(&db, 1, &template, "Utilisateur introuvable")
    ///     .await?;
    /// ```
    pub async fn get_or_404(
        &self,
        db: &DatabaseConnection,
        id: impl Into<<E::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType>,
        template: &Template,
        error_msg: &str,
    ) -> Result<E::Model, Response> {
        match self.get_optional(db, id).await {
            Ok(Some(entity)) => Ok(entity),
            Ok(None) => Err(template.render_404(error_msg)),
            Err(_) => Err(template.render_500("Erreur de base de données")),
        }
    }
}

// Implémentation Copy et Clone pour pouvoir utiliser Objects facilement
impl<E: EntityTrait> Copy for Objects<E> {}
impl<E: EntityTrait> Clone for Objects<E> {
    fn clone(&self) -> Self {
        *self
    }
}
