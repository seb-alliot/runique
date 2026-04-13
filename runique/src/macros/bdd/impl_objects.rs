//! Macro `impl_objects!` — adds the `objects` constant to a SeaORM entity.

/// Adds a Django-style objects manager to a SeaORM entity.
///
/// Generates a constant field `objects` allowing the syntax
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
