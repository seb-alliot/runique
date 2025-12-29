#[macro_export]
macro_rules! urlpatterns {
    (
        $($path:expr => $handler:expr, name = $name:expr) ,* $(,)?
    ) => {{
        let mut router = $crate::Router::new();

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
        let mut router = $crate::Router::new();
        $(
            router = router.route(
                $path,
                $handler
            );
        )*
        router
    }};
}
