use axum::{Router, routing::MethodRouter};

#[cfg(feature = "admin")]
use crate::{
    admin::router::AdminView, app::staging::admin_staging::DeclaredAdminRoute,
    auth::meta::ModelMeta,
};

/// Route declarations produced by [`crate::urlpatterns!`] when it contains
/// admin views.
///
/// Pass this value to `RuniqueAppBuilder::routes`. It keeps Runique-specific
/// route metadata until the application is assembled.
///
/// An admin view keeps its declared application path (no admin URL prefix),
/// but is served inside the admin authentication boundary. The handler
/// remains responsible for authorizing each data operation through the typed
/// gate:
///
/// ```rust,ignore
/// urlpatterns! {
///     admin "/blog/{id}/detail" => view! { blog_detail_custom },
///         name = "blog_detail_custom",
///         for_model = blog::Entity,
///         overrides = AdminView::Detail,
/// }
/// ```
pub struct Routes {
    pub(crate) router: Router,
    pub(crate) public_paths: Vec<String>,
    #[cfg(feature = "admin")]
    pub(crate) admin_routes: Vec<DeclaredAdminRoute>,
}

impl Routes {
    /// Used by `urlpatterns!`; construct route sets through the macro.
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            public_paths: Vec::new(),
            #[cfg(feature = "admin")]
            admin_routes: Vec::new(),
        }
    }

    /// Used by `urlpatterns!` to add an ordinary route.
    #[doc(hidden)]
    pub fn add_route(&mut self, path: impl AsRef<str>, handler: MethodRouter) {
        let path = path.as_ref();
        self.public_paths.push(path.to_string());
        self.router = std::mem::replace(&mut self.router, Router::new()).route(path, handler);
    }

    /// Adds an ordinary route while preserving the declaration metadata.
    pub fn route(mut self, path: impl AsRef<str>, handler: MethodRouter) -> Self {
        self.add_route(path, handler);
        self
    }

    /// Used by `urlpatterns!` to add a named ordinary route.
    #[doc(hidden)]
    pub fn add_named_route(
        &mut self,
        path: impl AsRef<str>,
        name: impl Into<String>,
        handler: MethodRouter,
    ) {
        let path = path.as_ref();
        crate::macros::routeur::register_url::register_pending(name, path);
        self.add_route(path, handler);
    }

    /// Merges another route set or an Axum router into this set.
    ///
    /// Admin declarations from another `Routes` value remain staged for the
    /// final application build.
    pub fn merge(mut self, other: impl Into<Routes>) -> Self {
        let other = other.into();
        self.router = std::mem::replace(&mut self.router, Router::new()).merge(other.router);
        self.public_paths.extend(other.public_paths);
        #[cfg(feature = "admin")]
        self.admin_routes.extend(other.admin_routes);
        self
    }

    /// The ordinary routes as an Axum router, for Runique's own assembly.
    #[cfg(feature = "admin")]
    pub(crate) fn into_router(self) -> Router {
        self.router
    }

    /// Used by `urlpatterns!` to retain an admin route declaration until build.
    #[cfg(feature = "admin")]
    #[doc(hidden)]
    pub fn add_admin_route<E: ModelMeta>(
        &mut self,
        path: impl Into<String>,
        name: impl Into<String>,
        handler: MethodRouter,
        overrides: AdminView,
    ) {
        let path = path.into();
        self.admin_routes.push(DeclaredAdminRoute {
            path: format!("/{}", path.trim_matches('/')),
            name: name.into(),
            model_key: E::KEY.to_string(),
            handler,
            overrides,
        });
    }
}

impl Default for Routes {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Router> for Routes {
    fn from(router: Router) -> Self {
        Self {
            router,
            public_paths: Vec::new(),
            #[cfg(feature = "admin")]
            admin_routes: Vec::new(),
        }
    }
}
