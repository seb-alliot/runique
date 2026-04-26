//! Definition of the `admin!` macro — binds SeaORM models, forms, and admin display options.

#[macro_export]
macro_rules! admin {
    // With configure block at the head (optional)
    (
        configure { $($_cfg:tt)* }
        $(
            $key:ident : $($model:ident)::+ => $form:path {
                title: $title:literal
                $(, create_form: $create_form_path:path)?
                $(, edit_form: $edit_form_path:path)?
                $(, list_display: [ $([$display_col:literal, $display_label:literal]),* $(,)? ])?
                $(, list_filter: [ $([$filter_col:literal, $filter_label:literal $(, $entry_limit:literal)?]),* $(,)? ])?
                $(,)?
            }
        )*
    ) => {
        // Compile-time verification — validates dev imports
        // If a type is missing → explicit compilation error
        $(
            const _: () = {
                fn _check_types() {
                    fn _model(_: &$($model)::+) {}
                    fn _form(_: &$form) {}
                }
            };
        )*
    };

    // Without configure block
    (
        $(
            $key:ident : $($model:ident)::+ => $form:path {
                title: $title:literal
                $(, create_form: $create_form_path:path)?
                $(, edit_form: $edit_form_path:path)?
                $(, list_display: [ $([$display_col:literal, $display_label:literal]),* $(,)? ])?
                $(, list_filter: [ $([$filter_col:literal, $filter_label:literal $(, $entry_limit:literal)?]),* $(,)? ])?
                $(,)?
            }
        )*
    ) => {
        // Compile-time verification — validates dev imports
        // If a type is missing → explicit compilation error
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
