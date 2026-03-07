#[macro_export]
macro_rules! admin {
    (
        $(
            $key:ident : $($model:ident)::+ => $form:ident {
                title: $title:literal ,
                permissions: [ $($perm:literal),* $(,)? ]
                $(, edit_form: $edit_form_path:path)?
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
                    $(fn _edit_form(_: &$edit_form_path) {})?
                }
            };
        )*
    };
}
