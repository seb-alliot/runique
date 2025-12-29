#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// Manager Django-like pour les queries
            /// on desactive l'avertissement car le nom est en minuscules
            /// exemple: Model.objects.filter(...)
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::orm::Objects<Self> = $crate::orm::Objects::new();
        }
    };
}
