#[macro_export]
macro_rules! urlpatterns {
    // Version avec NAME
    (
        $($path:expr => $handler:expr, name = $name:expr) ,* $(,)?
    ) => {{
        let mut router = $crate::Router::new();

        $(
            $crate::register_name_url($name, $path);
            router = router.route(
                $path,
                $handler // 👈 On capture toute l'expression ici
            );
        )*
        router
    }};

    // Version sans NAME
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

