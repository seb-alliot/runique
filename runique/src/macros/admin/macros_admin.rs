#[macro_export]
macro_rules! admin {
    (
        $(
            $key:ident : $($model:ident)::+ => $form:ident {
                title: $title:literal ,
                permissions: [ $($perm:literal),* $(,)? ]
                $(,)?
            }
        )*
    ) => {
        /// Construit l'AdminRegistry depuis les déclarations admin!{}
        ///
        /// Appelé par AdminStaging lors du build() pour enregistrer
        /// les ressources sans passer par le code généré.
        pub fn admin_config() -> $crate::admin::AdminRegistry {
            let mut registry = $crate::admin::AdminRegistry::new();

            $(
                registry.register(
                    $crate::admin::AdminResource::new(
                        stringify!($key),
                        stringify!($($model)::+),
                        stringify!($form),
                        $title,
                        vec![ $( $perm.to_string() ),* ],
                    )
                );
            )*

            registry
        }
    };
}
