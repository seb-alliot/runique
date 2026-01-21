#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// on desactive l'avertissement car le nom est en minuscules
            /// exemple: Model.objects.filter(...)
            #[allow(non_upper_case_globals)]
            pub const objects: $crate::data_base_runique::composant_data_base::Objects<Self> =
                $crate::data_base_runique::composant_data_base::Objects::new();
        }
    };
}
