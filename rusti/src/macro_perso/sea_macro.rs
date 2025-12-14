#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// Manager Django-like pour les queries
            pub const objects: $crate::orm::Objects<Self> = $crate::orm::Objects::new();
        }
    };
}