//! Définition de la macro `admin!` — lie modèles SeaORM, formulaires et options d'affichage admin.

#[macro_export]
macro_rules! admin {
    // Avec bloc configure en tête (optionnel)
    (
        configure { $($_cfg:tt)* }
        $(
            $key:ident : $($model:ident)::+ => $form:path {
                title: $title:literal ,
                permissions: [ $($perm:literal),* $(,)? ]
                $(, create_form: $create_form_path:path)?
                $(, edit_form: $edit_form_path:path)?
                $(, list_display: [ $([$display_col:literal, $display_label:literal]),* $(,)? ])?
                $(, list_filter: [ $([$filter_col:literal, $filter_label:literal $(, $entry_limit:literal)?]),* $(,)? ])?
                $(,)?
            }
        )*
    ) => {
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

    // Sans bloc configure
    (
        $(
            $key:ident : $($model:ident)::+ => $form:path {
                title: $title:literal ,
                permissions: [ $($perm:literal),* $(,)? ]
                $(, create_form: $create_form_path:path)?
                $(, edit_form: $edit_form_path:path)?
                $(, list_display: [ $([$display_col:literal, $display_label:literal]),* $(,)? ])?
                $(, list_filter: [ $([$filter_col:literal, $filter_label:literal $(, $entry_limit:literal)?]),* $(,)? ])?
                $(,)?
            }
        )*
    ) => {
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
