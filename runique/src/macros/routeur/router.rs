//! Macro `urlpatterns!` — keeps route declarations until application assembly.
//!
//! A declaration set can include admin views, which are separated from public
//! routes and mounted inside the admin boundary during `build()`.

pub use crate::macros::routeur::register_url::{
    add_urls, register_name_url, register_pending, reverse, reverse_with_parameters,
};

#[macro_export]
macro_rules! urlpatterns {
    (@parse $routes:ident;) => {};

    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty,
        overrides = $view:expr,
        $($rest:tt)*
    ) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };

    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty,
        overrides = $view:expr
    ) => {
        $routes.add_admin_route::<$model>($path, $name, $handler, $view);
    };

    // `overrides` left out: a standalone page.
    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty,
        $($rest:tt)*
    ) => {
        $routes.add_admin_route::<$model>(
            $path,
            $name,
            $handler,
            $crate::admin::AdminView::Standalone,
        );
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };

    (@parse $routes:ident;
        admin $path:expr => $handler:expr,
        name = $name:expr,
        for_model = $model:ty
    ) => {
        $routes.add_admin_route::<$model>(
            $path,
            $name,
            $handler,
            $crate::admin::AdminView::Standalone,
        );
    };

    (@parse $routes:ident; $path:expr => $handler:expr, name = $name:expr, $($rest:tt)*) => {
        $routes.add_named_route($path, $name, $handler);
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };

    (@parse $routes:ident; $path:expr => $handler:expr, name = $name:expr) => {
        $routes.add_named_route($path, $name, $handler);
    };

    (@parse $routes:ident; $path:expr => $handler:expr, $($rest:tt)*) => {
        $routes.add_route($path, $handler);
        $crate::urlpatterns!(@parse $routes; $($rest)*);
    };

    (@parse $routes:ident; $path:expr => $handler:expr) => {
        $routes.add_route($path, $handler);
    };

    ($($routes:tt)*) => {{
        let mut __runique_routes = $crate::macros::routeur::Routes::new();
        $crate::urlpatterns!(@parse __runique_routes; $($routes)*);
        __runique_routes
    }};
}
