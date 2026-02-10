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

        // Vérification compile-time — justifie les `use` du dev
        // Si un type est introuvable → erreur de compilation explicite
        $(
            const _: () = {
                fn _check_types() {
                    fn _model(_: &$($model)::+) {}
                    fn _form(_: &$form) {}
                }
            };
        )*
    };
}
