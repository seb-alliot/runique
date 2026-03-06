/// Ajoute un gestionnaire d'objets de style Django à une entité SeaORM.
///
/// Génère un champ constant `objects` qui permet la syntaxe
/// `Entity::objects.filter(...).all(&db).await`
///
#[doc = include_str!("../../../doc-tests/macro_db/impl_objects.md")]
#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::macros::bdd::objects::Objects<Self> =
                $crate::macros::bdd::objects::Objects::new();
        }
    };
}
