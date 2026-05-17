//! Definition of the `admin!` macro — binds SeaORM models, forms, and admin display options.

#[macro_export]
macro_rules! admin {
    // With configure block at the head (optional)
    (
        configure { $($_cfg:tt)* }
        $(
            $key:ident : $($model:ident)::+ => $form:path { $($body:tt)* }
        )*
    ) => {
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
            $key:ident : $($model:ident)::+ => $form:path { $($body:tt)* }
        )*
    ) => {
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
