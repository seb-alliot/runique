pub mod get_post;
pub mod register_name_url;

pub use register_name_url::{
    flush_pending_urls, register_name_url, register_pending, reverse, reverse_with_parameters,
};

#[macro_export]
macro_rules! urlpatterns {
    (
        $($path:expr => $handler:expr, name = $name:expr) ,* $(,)?
    ) => {{
        let mut router = $crate::axum::Router::new();

        $(
            $crate::macro_runique::router::register_pending($name, $path);

            router = router.route($path, $handler);
        )*
        router
    }};

    (
        $($path:expr => $handler:expr) ,* $(,)?
    ) => {{
        let mut router = $crate::axum::Router::new();

        $(
            router = router.route($path, $handler);
        )*
        router
    }};
}
