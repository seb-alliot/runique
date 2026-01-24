#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// Ajoute un gestionnaire d'objets de style Django à l'entité
            ///
            /// Cette macro ajoute un champ constant `objects` qui permet d'utiliser la syntaxe
            /// Django-like: `User::objects.filter(...).all(&db).await`
            ///
            /// # Exemple
            ///
            /// ```rust
            /// use runique::impl_objects;
            ///
            /// #[derive(Clone, Debug, DeriveEntityModel)]
            /// #[sea_orm(table_name = "users")]
            /// pub struct Model { /* ... */ }
            ///
            /// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            /// pub enum Relation {}
            ///
            /// impl ActiveModelBehavior for ActiveModel {}
            ///
            /// impl_objects!(Entity);  // Ajoute User::objects
            /// ```
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::db::Objects<Self> = $crate::db::Objects::new();
        }
    };
}
