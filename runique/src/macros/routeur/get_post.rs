//! Macro pour construire des routes avec GET/POST

#[macro_export]
macro_rules! view {
    (GET => $get_handler:expr, POST => $post_handler:expr) => {
        $crate::axum::routing::get($get_handler).post($post_handler)
    };

    (POST => $post_handler:expr, GET => $get_handler:expr) => {
        $crate::axum::routing::get($get_handler).post($post_handler)
    };

    (GET => $get_handler:expr) => {
        $crate::axum::routing::get($get_handler)
    };

    (POST => $post_handler:expr) => {
        $crate::axum::routing::post($post_handler)
    };
}
