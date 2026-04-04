//! Macros `tpl!` / `tpls!` — inclusion statique (`include_str!`) des templates depuis `runique/templates/`.

#[macro_export]
macro_rules! tpl {
    ($name:literal, $path:literal) => {
        ($name, include_str!(concat!("../../../templates/", $path)))
    };
}

#[macro_export]
macro_rules! tpls {
    ( $( ($name:literal, $path:literal) ),* $(,)? ) => {
        &[
            $(
                $crate::tpl!($name, $path),
            )*
        ] as &[(&str, &str)]
    };
}
