pub mod get_post;
pub mod register_name_url;

pub use register_name_url::{reverse, reverse_with_parameters};
#[macro_export]
macro_rules! urlpatterns {
    (
        $($path:expr => $handler:expr, name = $name:expr) ,* $(,)?
    ) => {{
        let mut router: $crate::axum::Router<std::sync::Arc<$crate::tera::Tera>> = $crate::axum::Router::new();

        $(
            $crate::register_name_url($name, $path);
            router = router.route(
                $path,
                $handler
            );
        )*
        router
    }};

    (
        $($path:expr => $handler:expr) , * $(,)?
    ) => {{
        let mut router: $crate::axum::Router<std::sync::Arc<$crate::tera::Tera>> = $crate::axum::Router::new();
        $(
            router = router.route(
                $path,
                $handler
            );
        )*
        router
    }};
}
