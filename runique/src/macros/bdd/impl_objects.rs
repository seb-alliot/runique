#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// Ajoute un gestionnaire d'objets de style Django à l'entité
            ///
            /// Cette macro ajoute un champ constant `objects` qui permet d'utiliser la syntaxe
            /// Django-like: `Entity::objects.filter(...).all(&db).await`
            ///
            /// # Exemple
            ///
            /// ```rust
            /// use sea_orm::entity::prelude::*;
            /// use runique::impl_objects;
            ///
            /// #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
            /// #[sea_orm(table_name = "users")]
            /// pub struct Model {
            ///     #[sea_orm(primary_key)]
            ///     pub id: i32,
            ///     pub username: String,
            /// }
            ///
            /// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            /// pub enum Relation {}
            ///
            /// impl ActiveModelBehavior for ActiveModel {}
            ///
            /// impl_objects!(Entity); // Ajoute Entity::objects
            /// ```
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::macros::bdd::objects::Objects<Self> =
                $crate::macros::bdd::objects::Objects::new();
        }
    };
}
